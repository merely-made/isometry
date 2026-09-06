-- Ash and Bells is a small first scene: a walkable approach, a broken tower,
-- and people who can be talked with before anyone reaches for steel.
-- The host supplies one deterministic draw. Locks are visible authoring
-- constraints, so the breach may be held east or west by the table.
function call_gen(request_json, entropy, request)
    local breach = "west"
    local locked = request.locks.breach
    if locked ~= nil and locked.type == "text" then
        if locked.value == "east" or locked.value == "west" then
            breach = locked.value
        end
    elseif entropy % 2 == 1 then
        breach = "east"
    end
    local breach_col = 8
    if breach == "east" then
        breach_col = 12
    end

    local nest_row = 6
    if entropy % 4 >= 2 then
        nest_row = 8
    end

    return {
        type = "campaign",
        campaign_json = [[
{
  "id":"watchtower:ash-and-bells",
  "name":"Ash and Bells",
  "world":{
    "factions":{
      "bellwardens":{"id":"bellwardens","name":"Bellwardens","tags":["watch","duty"],"claims":["watchtower:ruined-watchtower"]},
      "road-scavengers":{"id":"road-scavengers","name":"Road Scavengers","tags":["scavengers","survivors"],"claims":["watchtower:ruined-watchtower"]}
    },
    "places":{
      "watchtower:forest-region":{"id":"watchtower:forest-region","name":"Bellwood Reach","tags":["region","forest","frontier"],"map":"watchtower:forest-region","position":[5,4]},
      "watchtower:forest-entry":{"id":"watchtower:forest-entry","name":"The Forest Road","tags":["entry","forest","road"],"map":"watchtower:forest-entry","position":[0,4]},
      "watchtower:stream-clearing":{"id":"watchtower:stream-clearing","name":"Alder Stream Clearing","tags":["stream","clearing","local"],"map":"watchtower:stream-clearing","position":[10,4]},
      "watchtower:ruined-watchtower":{"id":"watchtower:ruined-watchtower","name":"The Ash-Bell Watchtower","tags":["ruin","frontier","local"],"map":"watchtower:ruined-watchtower","position":[5,1]}
    },
    "characters":{
      "brin":{"id":"brin","name":"Brin Tallow","tags":["scavenger","negotiator"],"faction":"road-scavengers","place":"watchtower:ruined-watchtower"},
      "otta":{"id":"otta","name":"Otta Reed","tags":["scavenger","lookout"],"faction":"road-scavengers","place":"watchtower:ruined-watchtower"},
      "elian":{"id":"elian","name":"Elian Voss","tags":["traveler","stranded"],"place":"watchtower:ruined-watchtower"},
      "mira":{"id":"mira","name":"Mira Vale","tags":["hero","negotiator"],"place":"watchtower:ruined-watchtower"}
    },
    "routes":{
      "forest-entry-region":{"id":"forest-entry-region","from":"watchtower:forest-entry","to":"watchtower:forest-region","tags":["road","forest"]},
      "stream-region":{"id":"stream-region","from":"watchtower:stream-clearing","to":"watchtower:forest-region","tags":["water","crossing"]},
      "tower-region":{"id":"tower-region","from":"watchtower:ruined-watchtower","to":"watchtower:forest-region","tags":["watch","road"]}
    },
    "laws":{},
    "history":[
      {"id":"bell-cracked","time":-12,"kind":"collapse","text":"The watchtower bell cracked when the old road burned, leaving its warning unfinished.","participants":["bellwardens"],"place":"watchtower:ruined-watchtower","tags":["ruin","bell"]},
      {"id":"scavengers-arrived","time":-2,"kind":"arrival","text":"Brin and Otta reached the tower before the road closed and made a camp among the fallen stones.","participants":["brin","otta","road-scavengers"],"place":"watchtower:ruined-watchtower","tags":["scavengers","camp"]}
    ],
    "storylets":{
      "parley-at-the-breach":{"key":"parley-at-the-breach","entry":"Brin raises an empty hand at the breach. The scavengers will share their water if the party helps Elian and leaves their camp standing.","tags":["negotiation","encounter","scavengers"],"roles":[{"key":"speaker","tags":["hero","negotiator"]},{"key":"scavenger","tags":["scavenger"]}],"effects":[{"type":"fact","fact":{"id":"watchtower:parley","kind":"outcome","text":"The scavengers lower their blades and point the traveler toward the old road.","tags":["negotiation","conclusion"]}}]},
      "steel-in-the-stones":{"key":"steel-in-the-stones","entry":"Otta's hand closes around a chipped blade. A shouted accusation, a drawn weapon, or a step toward the nest turns the ruin into a fight.","tags":["fight","encounter","scavengers"],"roles":[{"key":"challenger","tags":["hero"]},{"key":"lookout","tags":["lookout"]}],"effects":[{"type":"history","event":{"id":"watchtower:stone-fight","time":0,"kind":"skirmish","text":"The tower stones rang with a short, sharp fight over the stranded traveler and the nest.","participants":["brin","otta"],"place":"watchtower:ruined-watchtower","tags":["fight","conclusion"]}}]},
      "the-last-bell":{"key":"the-last-bell","entry":"With the stranded traveler safe and the tower beast faced, someone may climb to the broken bell and decide what warning the road hears next.","tags":["conclusion","encounter","watchtower"],"roles":[{"key":"climber","tags":["hero"]}],"effects":[{"type":"fact","fact":{"id":"watchtower:last-bell","kind":"outcome","text":"The next warning from the tower belongs to the people who survived it.","tags":["bell","conclusion"]}}]}
    }
  },
  "maps":[
    {"scale":"local","map":{
      "id":"watchtower:ruined-watchtower",
      "name":"Ash-Bell Watchtower",
      "width":15,
      "height":15,
      "default_ground":"forest-floor",
      "cells":[
        {"col":1,"row":7,"ground":"grass","elevation":0},{"col":2,"row":7,"ground":"grass","elevation":0},{"col":3,"row":7,"ground":"grass","elevation":0},{"col":4,"row":7,"ground":"grass","elevation":0},{"col":5,"row":7,"ground":"rubble","elevation":0},{"col":6,"row":7,"ground":"stone","elevation":0},{"col":7,"row":7,"ground":"stone","elevation":1},
        {"col":8,"row":4,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":4,"ground":"stone","prop":"wall","elevation":4},{"col":10,"row":4,"ground":"stone","prop":"wall","elevation":4},{"col":11,"row":4,"ground":"stone","prop":"wall","elevation":4},{"col":12,"row":4,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":5,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":5,"ground":"stone","elevation":5},{"col":10,"row":5,"ground":"stone","elevation":2},{"col":11,"row":5,"ground":"stone","elevation":2},{"col":12,"row":5,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":6,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":6,"ground":"stone","elevation":4},{"col":10,"row":6,"ground":"stone","elevation":2},{"col":11,"row":6,"ground":"stone","elevation":2},{"col":12,"row":6,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":7,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":7,"ground":"stone","elevation":2},{"col":10,"row":7,"ground":"stone","prop":"broken-bell","elevation":3},{"col":11,"row":7,"ground":"stone","elevation":2},{"col":12,"row":7,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":8,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":8,"ground":"stone","elevation":2},{"col":10,"row":8,"ground":"stone","elevation":2},{"col":11,"row":8,"ground":"stone","elevation":2},{"col":12,"row":8,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":9,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":9,"ground":"stone","elevation":2},{"col":10,"row":9,"ground":"stone","elevation":2},{"col":11,"row":9,"ground":"stone","elevation":2},{"col":12,"row":9,"ground":"stone","prop":"wall","elevation":4},
        {"col":8,"row":10,"ground":"stone","prop":"wall","elevation":4},{"col":9,"row":10,"ground":"stone","prop":"wall","elevation":4},{"col":10,"row":10,"ground":"stone","prop":"wall","elevation":4},{"col":11,"row":10,"ground":"stone","prop":"wall","elevation":4},{"col":12,"row":10,"ground":"stone","prop":"wall","elevation":4},
        {"col":]] .. breach_col .. [[,"row":7,"ground":"stone","prop":"tower-wall-]] .. breach .. [[","elevation":1},
        {"col":10,"row":6,"ground":"stone","prop":"nest-marker","elevation":4},
        {"col":10,"row":8,"ground":"stone","prop":"nest-marker","elevation":4},
        {"col":10,"row":5,"ground":"stone","elevation":6},{"col":10,"row":9,"ground":"stone","elevation":2},
        {"col":10,"row":2,"prop":"forest-tree"},
        {"col":1,"row":3,"prop":"forest-tree"},{"col":1,"row":5,"prop":"forest-tree"},{"col":0,"row":9,"prop":"forest-tree"},{"col":1,"row":11,"prop":"forest-tree"},{"col":3,"row":4,"prop":"forest-tree"},{"col":4,"row":1,"prop":"forest-tree"},{"col":5,"row":3,"prop":"forest-tree"},{"col":7,"row":1,"prop":"forest-tree"},{"col":9,"row":1,"prop":"forest-tree"},{"col":11,"row":1,"prop":"forest-tree"},{"col":14,"row":1,"prop":"forest-tree"},{"col":14,"row":6,"prop":"forest-tree"},{"col":14,"row":8,"prop":"forest-tree"},{"col":12,"row":13,"prop":"forest-tree"},{"col":10,"row":13,"prop":"forest-tree"},{"col":8,"row":13,"prop":"forest-tree"},{"col":4,"row":13,"prop":"forest-tree"},{"col":0,"row":13,"prop":"forest-tree"},{ "col":0,"row":1,"prop":"forest-tree"},{"col":3,"row":2,"prop":"forest-tree"},{"col":6,"row":2,"prop":"forest-tree"},{"col":13,"row":2,"prop":"forest-tree"},{"col":14,"row":4,"prop":"forest-tree"},{"col":2,"row":12,"prop":"forest-tree"},{"col":6,"row":12,"prop":"forest-tree"},{"col":13,"row":12,"prop":"forest-tree"},{"col":14,"row":10,"prop":"forest-tree"}
      ],
      "spawn_zones":[{"id":"party","cells":[{"col":1,"row":7},{"col":2,"row":7}]}],
      "transitions":[{"id":"road","at":{"col":1,"row":7},"target_map":"watchtower:forest-region","target_entry":"tower-road"}],
      "encounter_anchors":[{"id":"tower-beast-nest","at":{"col":10,"row":]] .. nest_row .. [[},"tags":["creature","tower-beast","nest"]}]
    },
    "inhabitants":[
      {"id":1,"name":"Mira Vale","sprite":"hero","at":{"col":2,"row":7},"system":"5e-srd","stats":{"str":14,"dex":12,"con":13,"int":10,"wis":10,"cha":12,"prof":2,"level":1,"hp_current":12,"hp_max":12,"ac":14,"attack_bonus":0,"speed":6,"sight":8,"will":12},"owner":"player"},
      {"id":2,"name":"Brin Tallow","sprite":"scavenger","at":{"col":9,"row":7},"system":"5e-srd","stats":{"str":10,"dex":14,"con":12,"int":11,"wis":12,"cha":15,"prof":2,"level":1,"hp_current":9,"hp_max":9,"ac":13,"attack_bonus":0,"speed":6,"sight":6,"will":13},"owner":"road-scavengers"},
      {"id":3,"name":"Otta Reed","sprite":"scavenger","at":{"col":11,"row":7},"system":"5e-srd","stats":{"str":12,"dex":16,"con":10,"int":10,"wis":13,"cha":9,"prof":2,"level":1,"hp_current":8,"hp_max":8,"ac":14,"attack_bonus":0,"speed":6,"sight":7,"will":12},"owner":"road-scavengers"},
      {"id":4,"name":"Elian Voss","sprite":"traveler","at":{"col":7,"row":10},"system":"5e-srd","stats":{"str":9,"dex":11,"con":10,"int":13,"wis":14,"cha":12,"prof":2,"level":1,"hp_current":5,"hp_max":5,"ac":11,"attack_bonus":0,"speed":6,"sight":6,"will":14}},
      {"id":5,"name":"Tower-beast","sprite":"tower-beast","at":{"col":10,"row":]] .. nest_row .. [[},"system":"5e-srd","stats":{"str":14,"dex":15,"con":12,"int":3,"wis":12,"cha":5,"prof":2,"level":1,"hp_current":18,"hp_max":18,"ac":14,"attack_bonus":0,"speed":8,"sight":8,"will":13}}
    ]
  },
    {"scale":"region","map":{
      "id":"watchtower:forest-region","name":"Bellwood Reach","width":11,"height":9,"default_ground":"forest-floor",
      "cells":[
        {"col":5,"row":0,"ground":"grass","prop":"forest-tree"},{"col":2,"row":1,"prop":"forest-tree"},{"col":8,"row":1,"prop":"forest-tree"},{"col":1,"row":3,"prop":"forest-tree"},{"col":9,"row":3,"prop":"forest-tree"},{"col":2,"row":7,"prop":"forest-tree"},{"col":8,"row":7,"prop":"forest-tree"},{"col":5,"row":8,"ground":"grass","prop":"forest-tree"},
        {"col":0,"row":4,"ground":"grass"},{"col":1,"row":4,"ground":"grass"},{"col":2,"row":4,"ground":"grass"},{"col":3,"row":4,"ground":"grass"},{"col":4,"row":4,"ground":"grass"},{"col":9,"row":4,"ground":"grass"},{"col":10,"row":4,"ground":"grass"},{"col":6,"row":0,"ground":"water"},{"col":7,"row":0,"ground":"water"},{"col":6,"row":1,"ground":"water"},{"col":7,"row":1,"ground":"water"},{"col":6,"row":2,"ground":"water"},{"col":7,"row":2,"ground":"water"},{"col":6,"row":3,"ground":"water"},{"col":7,"row":3,"ground":"water"},{ "col":5,"row":1,"ground":"grass"},{"col":5,"row":2,"ground":"grass"},{"col":5,"row":3,"ground":"grass"},{"col":5,"row":4,"ground":"stone"},{"col":5,"row":5,"ground":"grass"},{"col":5,"row":6,"ground":"grass"},{"col":5,"row":7,"ground":"grass"},
        {"col":6,"row":4,"ground":"stone"},{"col":6,"row":5,"ground":"water"},{"col":6,"row":6,"ground":"water"},{"col":7,"row":4,"ground":"stone"},{"col":7,"row":5,"ground":"water"},{"col":7,"row":6,"ground":"water"},{"col":8,"row":4,"ground":"stone"},{"col":8,"row":5,"ground":"stone"},{"col":8,"row":6,"ground":"stone"}
      ],
      "spawn_zones":[],
      "transitions":[
        {"id":"tower-road","at":{"col":5,"row":1},"target_map":"watchtower:ruined-watchtower","target_entry":"road"},
        {"id":"entry-road","at":{"col":0,"row":4},"target_map":"watchtower:forest-entry","target_entry":"entry-road"},
        {"id":"stream-road","at":{"col":10,"row":4},"target_map":"watchtower:stream-clearing","target_entry":"stream-road"}
      ],
      "encounter_anchors":[{"id":"wolf-trail","at":{"col":3,"row":3},"tags":["forest","trail"]}]
    }},
    {"scale":"local","map":{
      "id":"watchtower:forest-entry","name":"The Forest Road","width":9,"height":7,"default_ground":"forest-floor",
      "cells":[{"col":0,"row":3,"ground":"grass"},{"col":1,"row":3,"ground":"grass"},{"col":2,"row":3,"ground":"grass"},{"col":3,"row":3,"ground":"grass"},{"col":4,"row":3,"ground":"grass"},{"col":2,"row":1,"prop":"forest-tree"},{"col":6,"row":1,"prop":"forest-tree"},{"col":1,"row":5,"prop":"forest-tree"},{"col":7,"row":5,"prop":"forest-tree"}],
      "spawn_zones":[{"id":"party","cells":[{"col":0,"row":3},{"col":0,"row":4}]}],
      "transitions":[{"id":"entry-road","at":{"col":8,"row":3},"target_map":"watchtower:forest-region","target_entry":"entry-road"}],
      "encounter_anchors":[{"id":"road-sign","at":{"col":4,"row":3},"tags":["forest","clue"]}]
    }},
    {"scale":"local","map":{
      "id":"watchtower:stream-clearing","name":"Alder Stream Clearing","width":9,"height":7,"default_ground":"forest-floor",
      "cells":[{"col":0,"row":3,"ground":"grass"},{"col":1,"row":3,"ground":"grass"},{"col":2,"row":3,"ground":"stone"},{"col":3,"row":0,"ground":"water"},{"col":4,"row":0,"ground":"water"},{"col":5,"row":0,"ground":"water"},{"col":3,"row":1,"ground":"water"},{"col":4,"row":1,"ground":"water"},{"col":5,"row":1,"ground":"water"},{"col":3,"row":5,"ground":"water"},{"col":4,"row":5,"ground":"water"},{"col":5,"row":5,"ground":"water"},{"col":3,"row":6,"ground":"water"},{"col":4,"row":6,"ground":"water"},{"col":5,"row":6,"ground":"water"},{ "col":3,"row":2,"ground":"water"},{"col":4,"row":2,"ground":"water"},{"col":5,"row":2,"ground":"water"},{"col":3,"row":3,"ground":"stone"},{"col":4,"row":3,"ground":"stone"},{"col":5,"row":3,"ground":"stone"},{"col":3,"row":4,"ground":"water"},{"col":4,"row":4,"ground":"water"},{"col":5,"row":4,"ground":"water"},{"col":6,"row":3,"ground":"stone"},{"col":7,"row":3,"ground":"grass"},{"col":8,"row":3,"ground":"grass"},{"col":1,"row":1,"prop":"forest-tree"},{"col":7,"row":1,"prop":"forest-tree"},{"col":1,"row":5,"prop":"forest-tree"},{"col":7,"row":5,"prop":"forest-tree"}],
      "spawn_zones":[{"id":"party","cells":[{"col":0,"row":3},{"col":0,"row":4}]}],
      "transitions":[{"id":"stream-road","at":{"col":8,"row":3},"target_map":"watchtower:forest-region","target_entry":"stream-road"}],
      "encounter_anchors":[{"id":"alder-crossing","at":{"col":4,"row":3},"tags":["stream","crossing"]}]
    }}],
  "secrets":[],
  "rewards":[],
  "starting_map":"watchtower:ruined-watchtower",
  "final_storylet":"the-last-bell"
}
]]
    }
end





