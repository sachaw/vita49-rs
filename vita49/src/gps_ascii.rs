// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the ASCII GPS format
(ANSI/VITA-49.2-2017 section 9.4.7).
*/

use deku::prelude::*;

/// Base ASCII GPS data structure.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpsAscii {
    w1: u32,
    // Word count is wire-controlled and drives a `Vec::with_capacity`, so bound it
    // before the allocation: a whole VRT packet is at most 65535 32-bit words (the
    // 16-bit Packet Size field, Rule 5.1.1-10), so the ASCII payload cannot exceed
    // that. Without this, a crafted `num_words` reserves multi-GB (and overflows
    // `usize` on 32-bit targets) before hitting end-of-input.
    #[deku(assert = "*num_words <= 0xFFFF")]
    num_words: u32,
    #[deku(count = "num_words")]
    ascii: Vec<u32>,
}

impl GpsAscii {
    /// Gets the size of the ASCII GPS field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (((std::mem::size_of_val(&self.w1) + std::mem::size_of_val(&self.num_words))
            / std::mem::size_of::<u32>())
            + self.num_words as usize) as u16
    }
}
