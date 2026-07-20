// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the gain format
(ANSI/VITA-49.2-2017 section 9.5.3).

"In RF equipment such as tuners and receivers, the total gain
of the equipment is typically distributed to allow tradeoffs
between noise power and linearity. For such equipment, Stage 1
Gain conveys the front-end or RF gain, and Stage 2 Gain conveys
the back-end or IF gain. For equipment that does not require gain
distribution, Stage 1 Gain provides the gain of the device, and
Stage 2 Gain is set to zero."
*/

use deku::prelude::*;
use fixed::{types::extra::U7, FixedI16};
use std::fmt;

/// Base gain data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gain(i32);

impl Gain {
    /// Create a new `Gain` object given stage 1 and 2 gain in dB.
    pub fn new(stage_1_gain_db: f32, stage_2_gain_db: f32) -> Gain {
        // Go through `u16` so a negative stage 1 gain doesn't sign-extend
        // into (and clobber) the stage 2 half-word.
        let s1 = FixedI16::<U7>::from_num(stage_1_gain_db).to_bits() as u16 as i32;
        let s2 = FixedI16::<U7>::from_num(stage_2_gain_db).to_bits() as u16 as i32;
        Gain((s2 << 16) | s1)
    }

    /// Gets the size of the gain structure in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (std::mem::size_of_val(&self.0) / std::mem::size_of::<u32>()) as u16
    }

    /// Gets stage 1 gain (dB)
    pub fn stage_1_gain_db(&self) -> f32 {
        let s1 = (self.0 & 0xFFFF) as i16;
        FixedI16::<U7>::from_bits(s1).to_num()
    }

    /// Sets stage 1 gain (dB)
    pub fn set_stage_1_gain_db(&mut self, stage_1_gain_db: f32) {
        // Go through `u16` so a negative stage 1 gain doesn't sign-extend
        // into (and clobber) the stage 2 half-word.
        let s1 = FixedI16::<U7>::from_num(stage_1_gain_db).to_bits() as u16 as i32;
        self.0 = (self.0 & (0xFFFF_0000u32 as i32)) | s1
    }

    /// Gets stage 2 gain (dB)
    pub fn stage_2_gain_db(&self) -> f32 {
        let s2 = ((self.0 >> 16) & 0xFFFF) as i16;
        FixedI16::<U7>::from_bits(s2).to_num()
    }

    /// Sets stage 2 gain (dB)
    pub fn set_stage_2_gain_db(&mut self, stage_2_gain_db: f32) {
        let s2 = FixedI16::<U7>::from_num(stage_2_gain_db).to_bits() as u16 as i32;
        self.0 = (self.0 & 0x0000_FFFF) | (s2 << 16)
    }
}

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Stage 1: {} dB, Stage 2: {} dB",
            self.stage_1_gain_db(),
            self.stage_2_gain_db()
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::Gain;

    #[test]
    fn manipulate_gain() {
        let _ = env_logger::builder().is_test(true).try_init();
        use crate::prelude::*;
        let mut packet = Vrt::new_context_packet();
        let context = packet.payload_mut().context_mut().unwrap();
        let mut s1: f32 = 25.2;
        let mut s2: f32 = 0.23;
        let mut g = Gain::new(s1, s2);
        context.set_gain(Some(g));
        assert_relative_eq!(
            context.gain().unwrap().stage_1_gain_db(),
            s1,
            max_relative = 0.1
        );
        assert_relative_eq!(
            context.gain().unwrap().stage_2_gain_db(),
            s2,
            max_relative = 0.1
        );
        s1 = -20.5;
        s2 = -11.1;
        g.set_stage_1_gain_db(s1);
        g.set_stage_2_gain_db(s2);
        context.set_gain(Some(g));
        assert_relative_eq!(
            context.gain().unwrap().stage_1_gain_db(),
            s1,
            max_relative = 0.1
        );
        assert_relative_eq!(
            context.gain().unwrap().stage_2_gain_db(),
            s2,
            max_relative = 0.1
        );
    }

    #[test]
    fn negative_gain_round_trip() {
        let _ = env_logger::builder().is_test(true).try_init();
        // (stage 1, stage 2) combinations covering every sign mix. A negative
        // stage 1 gain must not sign-extend into the stage 2 half-word.
        let cases: [(f32, f32); 5] = [
            (-48.2, 12.5),
            (-48.2, 0.0),
            (12.5, -48.2),
            (-48.2, -1.5),
            (-0.5, -0.25),
        ];
        for (s1, s2) in cases {
            let g = Gain::new(s1, s2);
            assert_relative_eq!(g.stage_1_gain_db(), s1, max_relative = 0.01);
            assert_relative_eq!(g.stage_2_gain_db(), s2, max_relative = 0.01);
        }
    }

    #[test]
    fn negative_stage_1_setter_preserves_stage_2() {
        let _ = env_logger::builder().is_test(true).try_init();
        // Set stage 2 first, then a negative stage 1: stage 2 must survive.
        let mut g = Gain::new(0.0, 0.0);
        g.set_stage_2_gain_db(12.5);
        g.set_stage_1_gain_db(-48.2);
        assert_relative_eq!(g.stage_1_gain_db(), -48.2, max_relative = 0.01);
        assert_relative_eq!(g.stage_2_gain_db(), 12.5, max_relative = 0.01);
    }
}
