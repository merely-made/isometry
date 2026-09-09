//! Bridge between the async iroh session and the synchronous winit loop.
//!
//! The winit kernel owns the view-facing session data. A typed Armillary actor
//! owns Tokio and `HostNet` / `ClientNet`; it receives commands and emits
//! snapshots, campaign state, and status updates for the kernel to drain.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use armillary::{ActorHandle, Correlated, Emitter, RequestId, RequestIds, Wake};
use isometry_campaign::{CampaignStore, GenerationRecord};
use isometry_core::TokenId;
use isonetry::iroh_link::{ClientNet, HostNet};
use isonetry::{ActionIntent, GameEvent, GameSnapshot};
use muniment::Journal;

/// Which side of the session this process runs.
pub enum Role {
    /// The DM: authoritative, prints a join ticket.
    Host {
        state: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
    },
    /// A player: dials the host's ticket, announcing a name.
    Client { ticket: String, name: String },
}

enum BridgeCommand {
    Event(GameEvent),
    Travel {
        request: RequestId,
        token: TokenId,
        crossing: isonetry::RequestId,
        party: String,
    },
    /// A *client* asking the host to resolve an action. Carries no verdict.
    Action(ActionIntent),
    Campaign {
        request: RequestId,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
        party: Option<String>,
    },
    Storylet {
        request: RequestId,
        key: String,
        item_owner: Option<TokenId>,
    },
    FactionTurn {
        request: RequestId,
        moves: Vec<isometry_campaign::FactionMove>,
    },
    Whisper {
        to: String,
        text: String,
    },
}

enum BridgeUpdate {
    HostReady {
        ticket: String,
        snapshot: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
    },
    HostState {
        snapshot: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
        players: Vec<String>,
    },
    ClientState(GameSnapshot),
    /// Client action requests the host must adjudicate. They surface here rather
    /// than being answered inside the session, because `isonetry` is
    /// rules-blind: only the app holds a `System` that can say whether you hit.
    ActionIntents(Vec<ActionIntent>),
    Whispers(Vec<(String, String)>),
    CampaignFinished(Correlated<Result<(), String>>),
    Failed(String),
}

/// The winit-thread handle to the background session actor.
pub struct NetBridge {
    actor: ActorHandle<BridgeCommand>,
    updates: Receiver<BridgeUpdate>,
    snapshot: Option<GameSnapshot>,
    campaign: Option<CampaignStore>,
    history: Option<Journal<GameEvent>>,
    version: u64,
    ticket: Option<String>,
    inbox: Vec<(String, String)>,
    players: Vec<String>,
    request_ids: RequestIds,
    campaign_outcomes: Vec<Correlated<Result<(), String>>>,
    failure: Option<String>,
    action_intents: Vec<ActionIntent>,
}

impl NetBridge {
    /// Spawn the session actor.
    ///
    /// `wake` is the host's own wake handle, in the callback shape Armillary
    /// takes. The bridge used to be polled from an idle tick with a no-op wake;
    /// now the actor schedules one drain turn (`after_wake`) and one redraw
    /// when it actually has something, so a still table costs nothing and a
    /// peer's move arrives without waiting out a poll interval.
    pub fn spawn(role: Role, wake: Wake) -> Self {
        let (actor, updates) =
            armillary::spawn_named("isometry-session", wake, move |commands, out| {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("session runtime");
                rt.block_on(run(role, commands, out));
            });
        Self {
            actor,
            updates,
            snapshot: None,
            campaign: None,
            history: None,
            version: 0,
            ticket: None,
            inbox: Vec::new(),
            players: Vec::new(),
            request_ids: RequestIds::default(),
            campaign_outcomes: Vec::new(),
            failure: None,
            action_intents: Vec::new(),
        }
    }

    /// Drain actor updates on the winit thread. Returns true when a new
    /// snapshot was accepted and the view needs rebuilding.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.updates.try_recv() {
                Ok(BridgeUpdate::HostReady {
                    ticket,
                    snapshot,
                    campaign,
                    history,
                }) => {
                    self.ticket = Some(ticket);
                    self.campaign = Some(campaign);
                    self.history = Some(history);
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::HostState {
                    snapshot,
                    campaign,
                    history,
                    players,
                }) => {
                    self.campaign = Some(campaign);
                    self.history = Some(history);
                    self.players = players;
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::ClientState(snapshot)) => {
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::ActionIntents(mut intents)) => {
                    self.action_intents.append(&mut intents);
                    changed = true;
                },
                Ok(BridgeUpdate::Whispers(mut whispers)) => self.inbox.append(&mut whispers),
                Ok(BridgeUpdate::CampaignFinished(outcome)) => self.campaign_outcomes.push(outcome),
                Ok(BridgeUpdate::Failed(error)) => self.failure = Some(error),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        changed
    }

    fn set_snapshot(&mut self, snapshot: GameSnapshot) {
        self.snapshot = Some(snapshot);
        self.version = self.version.wrapping_add(1);
    }

    /// Queue a local game event for the session actor.
    pub fn submit(&self, event: GameEvent) {
        let _ = self.actor.command(BridgeCommand::Event(event));
    }

    pub fn travel(
        &mut self,
        token: TokenId,
        crossing: isonetry::RequestId,
        party: String,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Travel {
                request,
                token,
                crossing,
                party,
            })
            .then_some(request)
    }

    /// Ask the host to resolve an action (client side). The client decides
    /// nothing: it names an actor, a victim and an action, and waits.
    pub fn submit_action(&self, intent: ActionIntent) {
        let _ = self.actor.command(BridgeCommand::Action(intent));
    }

    /// Drain client action requests awaiting adjudication (host side).
    pub fn take_action_intents(&mut self) -> Vec<ActionIntent> {
        std::mem::take(&mut self.action_intents)
    }

    pub fn commit_campaign(
        &mut self,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
        party: Option<String>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Campaign {
                request,
                record,
                item_owner,
                party,
            })
            .then_some(request)
    }

    /// Ask the host to play a storylet (session path). Its effects replicate.
    pub fn commit_storylet(
        &mut self,
        key: String,
        item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Storylet {
                request,
                key,
                item_owner,
            })
            .then_some(request)
    }

    /// Ask the host to commit a downtime faction tick (session path). Each kept
    /// move's world events replicate to every peer.
    pub fn commit_faction_turn(
        &mut self,
        moves: Vec<isometry_campaign::FactionMove>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::FactionTurn { request, moves })
            .then_some(request)
    }

    pub fn take_campaign_outcomes(&mut self) -> Vec<Correlated<Result<(), String>>> {
        std::mem::take(&mut self.campaign_outcomes)
    }

    /// Host: send a directed whisper to a named player.
    pub fn whisper(&self, to: String, text: String) {
        let _ = self.actor.command(BridgeCommand::Whisper { to, text });
    }

    /// Client: take whispers received since the last call.
    pub fn take_whispers(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.inbox)
    }

    /// Host: connected player names (whisper targets).
    pub fn players(&self) -> Vec<String> {
        self.players.clone()
    }

    /// The current change version; the UI redraws when it advances.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The latest replicated snapshot, if one has arrived.
    pub fn latest(&self) -> Option<GameSnapshot> {
        self.snapshot.clone()
    }

    /// Host-only GM state as of the latest session poll.
    pub fn campaign(&self) -> Option<CampaignStore> {
        self.campaign.clone()
    }

    pub fn history(&self) -> Option<Journal<GameEvent>> {
        self.history.clone()
    }

    /// The host's join ticket, once bound.
    pub fn ticket(&self) -> Option<String> {
        self.ticket.clone()
    }

    /// A background bind/join failure. Reading clears the pending message.
    pub fn take_failure(&mut self) -> Option<String> {
        self.failure.take()
    }
}

async fn run(role: Role, commands: Receiver<BridgeCommand>, out: Emitter<BridgeUpdate>) {
    match role {
        Role::Host {
            state,
            campaign,
            history,
        } => run_host(state, campaign, history, commands, out).await,
        Role::Client { ticket, name } => run_client(ticket, name, commands, out).await,
    }
}

fn drain_commands(commands: &Receiver<BridgeCommand>) -> Result<Vec<BridgeCommand>, ()> {
    let mut drained = Vec::new();
    loop {
        match commands.try_recv() {
            Ok(command) => drained.push(command),
            Err(TryRecvError::Empty) => return Ok(drained),
            Err(TryRecvError::Disconnected) => return Err(()),
        }
    }
}

async fn run_host(
    state: GameSnapshot,
    campaign: CampaignStore,
    history: Journal<GameEvent>,
    commands: Receiver<BridgeCommand>,
    out: Emitter<BridgeUpdate>,
) {
    let host = match HostNet::bind_with_history(state, campaign, history).await {
        Ok(host) => host,
        Err(error) => {
            out.emit(BridgeUpdate::Failed(format!("host bind failed: {error}")));
            return;
        },
    };
    host.spawn_accept();
    let ticket = host.ticket().await;
    println!("[isometry] hosting. share this ticket to join:\n\n  {ticket}\n");
    out.emit(BridgeUpdate::HostReady {
        ticket,
        snapshot: host.snapshot().await,
        campaign: host.campaign().await,
        history: host.history().await,
    });

    let mut last_seq = host.seq().await;
    let mut last_players = Vec::new();
    loop {
        let commands = match drain_commands(&commands) {
            Ok(commands) => commands,
            Err(()) => break,
        };
        for command in commands {
            match command {
                BridgeCommand::Event(event) => host.local_event(event).await,
                // The DM resolves its own swings directly (it *is* the rules
                // system), so it never routes one through here. Handled anyway,
                // as the same request a player would send.
                BridgeCommand::Action(intent) => {
                    out.emit(BridgeUpdate::ActionIntents(vec![intent]));
                },
                BridgeCommand::Travel {
                    request,
                    token,
                    crossing,
                    party,
                } => {
                    let result = host
                        .commit_transition_for_party(token, crossing, &party)
                        .await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::Campaign {
                    request,
                    record,
                    item_owner,
                    party,
                } => {
                    let result = match party {
                        Some(party) => {
                            host.commit_campaign_for_party(record, item_owner, &party)
                                .await
                        },
                        None => host.commit_campaign(record, item_owner).await,
                    };
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::Storylet {
                    request,
                    key,
                    item_owner,
                } => {
                    // Reuse the campaign-outcome channel: both are one-shot
                    // "commit this, tell me if it took" results.
                    let result = host.commit_storylet(&key, item_owner).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::FactionTurn { request, moves } => {
                    // Same one-shot "commit this, tell me if it took" channel.
                    let result = host.commit_faction_turn(moves).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::Whisper { to, text } => host.whisper("dm", &to, &text).await,
            }
        }

        let intents = host.take_action_intents().await;
        if !intents.is_empty() {
            out.emit(BridgeUpdate::ActionIntents(intents));
        }

        let seq = host.seq().await;
        let players = host.player_names().await;
        if seq != last_seq || players != last_players {
            last_seq = seq;
            last_players = players.clone();
            out.emit(BridgeUpdate::HostState {
                snapshot: host.snapshot().await,
                campaign: host.campaign().await,
                history: host.history().await,
                players,
            });
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}

async fn run_client(
    ticket: String,
    name: String,
    commands: Receiver<BridgeCommand>,
    out: Emitter<BridgeUpdate>,
) {
    let client = match ClientNet::join(&ticket, &name).await {
        Ok(client) => client,
        Err(error) => {
            out.emit(BridgeUpdate::Failed(format!("join failed: {error}")));
            return;
        },
    };
    println!("[isometry] joined session as {name}; replaying host log.");
    let mut last_applied = u64::MAX;
    loop {
        let commands = match drain_commands(&commands) {
            Ok(commands) => commands,
            Err(()) => break,
        };
        for command in commands {
            match command {
                BridgeCommand::Event(event) => {
                    let _ = client.intent(event).await;
                },
                // The player asks; the host answers. Nothing about the outcome
                // travels with this.
                BridgeCommand::Action(intent) => {
                    let _ = client.action(intent).await;
                },
                BridgeCommand::Travel { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("travel is ruled by the host".to_owned()),
                    )));
                },
                BridgeCommand::Campaign { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("campaign commits require the host".to_owned()),
                    )));
                },
                BridgeCommand::Storylet { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("storylets are played by the host".to_owned()),
                    )));
                },
                BridgeCommand::FactionTurn { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("faction turns are run by the host".to_owned()),
                    )));
                },
                BridgeCommand::Whisper { .. } => {},
            }
        }

        let applied = client.applied().await;
        if applied != last_applied {
            if let Some(snapshot) = client.state().await {
                last_applied = applied;
                out.emit(BridgeUpdate::ClientState(snapshot));
            }
        }
        let whispers = client.take_whispers().await;
        if !whispers.is_empty() {
            out.emit(BridgeUpdate::Whispers(whispers));
        }
        tokio::time::sleep(Duration::from_millis(60)).await;
    }
}

#[cfg(test)]
mod tests;
