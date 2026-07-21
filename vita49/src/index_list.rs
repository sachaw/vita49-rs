// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
The Index List field (ANSI/VITA-49.2-2017 §9.3.2, CIF1 bit 7): a packed list of
index values selecting a subset of the Records of an Array-of-Records field.
*/

use deku::prelude::*;

/// Width of the packed entries in an [`IndexList`] (Table 9.3.2-1).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntrySize {
    /// 8-bit entries, four packed per 32-bit word.
    Bits8,
    /// 16-bit entries, two packed per 32-bit word.
    Bits16,
    /// 32-bit entries, one per word.
    Bits32,
}

impl EntrySize {
    /// The 4-bit Entry Size code (Table 9.3.2-1), from bits 31..28 of header word 2.
    fn from_code(code: u32) -> Option<EntrySize> {
        match code & 0xF {
            0b0001 => Some(EntrySize::Bits8),
            0b0010 => Some(EntrySize::Bits16),
            0b0100 => Some(EntrySize::Bits32),
            _ => None,
        }
    }
    fn code(self) -> u32 {
        match self {
            EntrySize::Bits8 => 0b0001,
            EntrySize::Bits16 => 0b0010,
            EntrySize::Bits32 => 0b0100,
        }
    }
    fn bits(self) -> u32 {
        match self {
            EntrySize::Bits8 => 8,
            EntrySize::Bits16 => 16,
            EntrySize::Bits32 => 32,
        }
    }
    /// Entries packed per 32-bit word.
    fn per_word(self) -> usize {
        (32 / self.bits()) as usize
    }
}

/// The Index List field (§9.3.2): a two-word header (total size, and the entry
/// size + entry count) followed by the packed index entries. Entries are packed
/// big-endian, most-significant subfield first (Rule 9.3.2-7), and the final word
/// is zero-padded.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexList {
    // Total field size in words. Bounded before the `count` below drives a
    // `Vec::with_capacity`: a whole packet is at most 65535 words (Rule 5.1.1-10),
    // so the field cannot exceed that.
    #[deku(assert = "*total_size_words <= 0xFFFF")]
    total_size_words: u32,
    // Entry Size [31:28] | Reserved [27:20] | # entries [19:0].
    header2: u32,
    #[deku(count = "(*total_size_words).saturating_sub(2) as usize")]
    words: Vec<u32>,
}

impl IndexList {
    /// Build an Index List from index values packed at the given entry size.
    ///
    /// Values wider than the entry size are truncated to its low bits. The final
    /// packed word is zero-padded (Rule 9.3.2-6).
    pub fn new(entries: &[u32], entry_size: EntrySize) -> IndexList {
        let per_word = entry_size.per_word();
        let bits = entry_size.bits();
        let mask = if bits == 32 {
            u32::MAX
        } else {
            (1 << bits) - 1
        };
        let num_words = entries.len().div_ceil(per_word);
        let mut words = vec![0u32; num_words];
        for (i, &e) in entries.iter().enumerate() {
            // Most-significant subfield first within the word (big-endian order).
            let shift = bits * (per_word as u32 - 1 - (i % per_word) as u32);
            words[i / per_word] |= (e & mask) << shift;
        }
        IndexList {
            total_size_words: (2 + num_words) as u32,
            header2: (entry_size.code() << 28) | ((entries.len() as u32) & 0xF_FFFF),
            words,
        }
    }

    /// The entry size code, or `None` if the field uses a reserved code.
    pub fn entry_size(&self) -> Option<EntrySize> {
        EntrySize::from_code(self.header2 >> 28)
    }

    /// The number of entries (header word 2, bits 19..0).
    pub fn num_entries(&self) -> u32 {
        self.header2 & 0xF_FFFF
    }

    /// The unpacked index values. Empty if the entry size is a reserved code.
    pub fn entries(&self) -> Vec<u32> {
        let Some(entry_size) = self.entry_size() else {
            return Vec::new();
        };
        let per_word = entry_size.per_word();
        let bits = entry_size.bits();
        let mask = if bits == 32 {
            u32::MAX
        } else {
            (1 << bits) - 1
        };
        // Cap by both the declared count and what the words actually hold.
        let n = (self.num_entries() as usize).min(self.words.len() * per_word);
        (0..n)
            .map(|i| {
                let shift = bits * (per_word as u32 - 1 - (i % per_word) as u32);
                (self.words[i / per_word] >> shift) & mask
            })
            .collect()
    }

    /// Size of the whole field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        self.total_size_words as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn roundtrip(entries: &[u32], size: EntrySize, expect_words: u16) {
        let field = IndexList::new(entries, size);
        assert_eq!(field.size_words(), expect_words);
        assert_eq!(field.entry_size(), Some(size));
        assert_eq!(field.entries(), entries);

        let mut packet = Vrt::new_context_packet();
        packet
            .payload_mut()
            .context_mut()
            .unwrap()
            .set_index_list(Some(field.clone()));
        packet.update_packet_size();

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        let got = parsed.payload().context().unwrap().index_list().unwrap();
        assert_eq!(got, &field);
        assert_eq!(got.entries(), entries);
    }

    #[test]
    fn index_list_packs_and_round_trips_at_each_width() {
        // 32-bit: 2 header + 3 words.
        roundtrip(
            &[0x1234_5678, 0xDEAD_BEEF, 0x0000_0001],
            EntrySize::Bits32,
            5,
        );
        // 16-bit: 5 entries -> ceil(5/2)=3 words (last half-padded) + 2 header.
        roundtrip(&[1, 2, 3, 4, 5], EntrySize::Bits16, 5);
        // 8-bit: 6 entries -> ceil(6/4)=2 words (last 3/4 padded) + 2 header.
        roundtrip(&[10, 20, 30, 40, 50, 60], EntrySize::Bits8, 4);
    }
}
