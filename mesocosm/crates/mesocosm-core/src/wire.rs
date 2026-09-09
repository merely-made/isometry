// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Framing for everything this game writes for somebody else to read.
//!
//! The wing couples its vessels **by data, not by types**, which trades a
//! compile error for a runtime one. The reader therefore has to be the thing
//! that notices, and it has to notice *before* it decodes.
//!
//! # Why the header is outside the payload
//!
//! Postcard is positional: no field tags, no self-description. A version field
//! *inside* the serialized struct is unreachable exactly when it is needed —
//! once the layout changes, the decoder cannot get to the field that would have
//! told it to refuse. It fails as a malformed decode, or worse, succeeds and
//! returns plausible nonsense.
//!
//! So every artifact starts with a fixed 10-byte header that never changes
//! shape: an 8-byte magic naming the schema, and a little-endian `u16` version.
//! Both are readable without decoding anything, which is what makes
//! [`WireError::UnknownVersion`] a diagnosis rather than a guess.
//!
//! One framing, used by every schema, so the refusal contract is written once
//! and every reader in the wing behaves the same way.

use serde::{Serialize, de::DeserializeOwned};

/// Magic plus a little-endian `u16`.
pub const HEADER_LEN: usize = 10;

/// Why framed bytes could not be read. Every variant is a refusal rather than
/// a fallback: a reader that cannot tell what it is holding must say so.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    /// Fewer bytes than a header.
    TooShort { got: usize },
    /// The magic does not match the schema being read. These bytes are
    /// something else — possibly a valid artifact of a different kind.
    WrongSchema { found: [u8; 8], expected: [u8; 8] },
    /// The magic matched, so this *is* the right kind of artifact, written by
    /// a build that does not agree with this one about the payload's shape.
    UnknownVersion { found: u16, expected: u16 },
    /// Header good, payload not: truncated or corrupt.
    Malformed,
    /// The payload decoded and then failed the schema's own coherence check.
    /// Carried here so a reader has one error type to handle.
    Inconsistent,
    /// The value could not be serialized.
    Encode,
}

/// Writes `value` behind a schema header.
pub fn frame<T: Serialize>(magic: [u8; 8], version: u16, value: &T) -> Result<Vec<u8>, WireError> {
    let payload = postcard::to_allocvec(value).map_err(|_| WireError::Encode)?;
    let mut bytes = Vec::with_capacity(HEADER_LEN + payload.len());
    bytes.extend_from_slice(&magic);
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

/// Reads bytes written by [`frame`], refusing anything it cannot vouch for.
///
/// Order matters and is the whole point: magic, then version, then payload. A
/// newer writer is diagnosed without the payload ever being touched.
pub fn unframe<T: DeserializeOwned>(
    magic: [u8; 8],
    version: u16,
    bytes: &[u8],
) -> Result<T, WireError> {
    let (found, payload) = split(magic, bytes)?;
    if found != version {
        return Err(WireError::UnknownVersion {
            found,
            expected: version,
        });
    }
    postcard::from_bytes(payload).map_err(|_| WireError::Malformed)
}

/// Reads the header without decoding the payload, for a reader that wants to
/// know what it is holding before deciding whether to hold it.
pub fn peek(magic: [u8; 8], bytes: &[u8]) -> Result<u16, WireError> {
    split(magic, bytes).map(|(version, _)| version)
}

fn split(magic: [u8; 8], bytes: &[u8]) -> Result<(u16, &[u8]), WireError> {
    if bytes.len() < HEADER_LEN {
        return Err(WireError::TooShort { got: bytes.len() });
    }
    let found: [u8; 8] = bytes[..8].try_into().expect("checked length");
    if found != magic {
        return Err(WireError::WrongSchema {
            found,
            expected: magic,
        });
    }
    Ok((
        u16::from_le_bytes([bytes[8], bytes[9]]),
        &bytes[HEADER_LEN..],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAGIC: [u8; 8] = *b"TESTWIRE";
    const OTHER: [u8; 8] = *b"OTHERSCH";

    #[test]
    fn framed_values_round_trip() {
        let bytes = frame(MAGIC, 0, &(7u32, "hello".to_string())).unwrap();
        let back: (u32, String) = unframe(MAGIC, 0, &bytes).unwrap();
        assert_eq!(back, (7, "hello".to_string()));
    }

    #[test]
    fn a_short_read_is_refused_first() {
        assert_eq!(
            unframe::<u32>(MAGIC, 0, b"TEST"),
            Err(WireError::TooShort { got: 4 })
        );
    }

    #[test]
    fn another_schema_is_named_rather_than_mis_decoded() {
        // A valid artifact of the wrong kind. The reader says which it wanted
        // and which it got, because "malformed" would send someone hunting a
        // corruption bug that is not there.
        let bytes = frame(OTHER, 0, &1u32).unwrap();
        assert_eq!(
            unframe::<u32>(MAGIC, 0, &bytes),
            Err(WireError::WrongSchema {
                found: OTHER,
                expected: MAGIC
            })
        );
    }

    #[test]
    fn a_newer_writer_is_diagnosed_without_decoding() {
        // The case an in-payload version field cannot handle: the layout
        // changed, so the payload is undecodable by this build. The header
        // still answers.
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&9u16.to_le_bytes());
        bytes.extend_from_slice(&[0xff; 24]);

        assert_eq!(
            unframe::<u32>(MAGIC, 0, &bytes),
            Err(WireError::UnknownVersion {
                found: 9,
                expected: 0
            })
        );
    }

    #[test]
    fn a_truncated_payload_is_malformed_not_a_version_problem() {
        let bytes = frame(MAGIC, 0, &"a reasonably long string".to_string()).unwrap();
        assert_eq!(
            unframe::<String>(MAGIC, 0, &bytes[..HEADER_LEN + 2]),
            Err(WireError::Malformed)
        );
    }

    #[test]
    fn peeking_reads_the_version_of_a_payload_we_cannot_decode() {
        let bytes = frame(MAGIC, 3, &"anything".to_string()).unwrap();
        assert_eq!(peek(MAGIC, &bytes), Ok(3));
        assert_eq!(
            peek(OTHER, &bytes),
            Err(WireError::WrongSchema {
                found: MAGIC,
                expected: OTHER
            })
        );
    }
}
