// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
State temporal attributes (ANSI/VITA-49.2-2017 §9.7.2): Age of Current State and
Shelf Life of Current State.
*/

use crate::packet_header::{PrologueCtx, Tsf, Tsi};
use deku::prelude::*;

/// A State Temporal Attribute (§9.7.2) — a duration whose wire format follows the
/// packet **prologue** TSI/TSF rather than a local indicator (Rules 9.7.2.1-2/-3,
/// 9.7.2.2-2/-3). Used for Age of Current State (§9.7.2.1) and Shelf Life of
/// Current State (§9.7.2.2). The integer-seconds word is present iff TSI ≠ Null;
/// the 64-bit fractional value is present iff TSF ≠ Null — so the field is 0, 1, 2
/// or 3 words (Table 9.7-1).
///
/// The fractional value is modeled as the §9.7 Fractional-Time data type: 64-bit
/// two's-complement with a 1-femtosecond LSB (Rule 9.7-1/-2, Observation 9.7-6),
/// consistent with every other temporal-category field. (The Rule cross-reference
/// to §5.1.4.2 could instead be read as the picosecond Fractional *Timestamp*; the
/// wire width is 2 words either way, only the LSB/signedness of the accessor
/// differs.)
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, prologue: PrologueCtx"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateTime {
    #[deku(cond = "prologue.tsi != Tsi::Null")]
    integer_seconds: Option<u32>,
    #[deku(cond = "prologue.tsf != Tsf::Null")]
    fractional_femtoseconds: Option<i64>,
}

impl StateTime {
    /// Build a state-temporal value from its optional integer-seconds and
    /// fractional-femtoseconds parts.
    ///
    /// Which parts are present **must** match the packet prologue's TSI/TSF
    /// (integer present iff TSI ≠ Null, fractional present iff TSF ≠ Null); a
    /// mismatch will not round-trip on the wire.
    pub fn new(seconds: Option<u32>, femtoseconds: Option<i64>) -> StateTime {
        StateTime {
            integer_seconds: seconds,
            fractional_femtoseconds: femtoseconds,
        }
    }

    /// The integer-seconds part of the duration, present iff the prologue TSI is set.
    pub fn seconds(&self) -> Option<u32> {
        self.integer_seconds
    }

    /// The fractional part in femtoseconds (1 fs LSB, two's-complement), present
    /// iff the prologue TSF is set.
    pub fn femtoseconds(&self) -> Option<i64> {
        self.fractional_femtoseconds
    }

    /// Size of this field in 32-bit words (0, 1, 2 or 3 per the prologue TSI/TSF).
    pub fn size_words(&self) -> u16 {
        u16::from(self.integer_seconds.is_some())
            + 2 * u16::from(self.fractional_femtoseconds.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::StateTime;
    use crate::prelude::*;

    /// Round-trip Age + Shelf Life through a context packet at a given prologue
    /// TSI/TSF, asserting the field's wire width follows the prologue: an integer
    /// word iff TSI != Null, a 64-bit (2-word) fractional value iff TSF != Null.
    fn roundtrip(tsi: Tsi, tsf: Tsf, expect_words: u16) {
        let secs = (tsi != Tsi::Null).then_some(42u32);
        let fs = (tsf != Tsf::Null).then_some(-1_234_567_890i64);
        let value = StateTime::new(secs, fs);
        assert_eq!(value.size_words(), expect_words, "tsi={tsi:?} tsf={tsf:?}");

        let mut packet = Vrt::new_context_packet();
        packet
            .set_integer_timestamp((tsi != Tsi::Null).then_some(1), tsi)
            .unwrap();
        packet
            .set_fractional_timestamp((tsf != Tsf::Null).then_some(2), tsf)
            .unwrap();
        {
            let ctx = packet.payload_mut().context_mut().unwrap();
            ctx.set_age(Some(value));
            ctx.set_shelf_life(Some(value));
        }
        packet.update_packet_size();

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        let ctx = parsed.payload().context().unwrap();
        assert_eq!(ctx.age(), Some(&value));
        assert_eq!(ctx.shelf_life(), Some(&value));
        assert_eq!(ctx.age().unwrap().seconds(), secs);
        assert_eq!(ctx.age().unwrap().femtoseconds(), fs);
    }

    #[test]
    fn age_and_shelf_life_size_follows_prologue() {
        roundtrip(Tsi::Null, Tsf::Null, 0); // degenerate: field present, 0 words
        roundtrip(Tsi::Utc, Tsf::Null, 1); // integer seconds only
        roundtrip(Tsi::Null, Tsf::RealTimePs, 2); // fractional only
        roundtrip(Tsi::Utc, Tsf::RealTimePs, 3); // both
    }
}
