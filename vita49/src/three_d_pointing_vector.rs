// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
The 3-D Pointing Vector Structure — the Array-of-Records form of the 3-D Pointing
Vector field (ANSI/VITA-49.2-2017 §9.4.1.2, CIF1 bit 28). Each record carries a
required Pointing Vector word (§9.4.1.1) and an optional Index/Reference/Beam word
(§9.4.1.5).
*/

use deku::prelude::*;
use fixed::{types::extra::U7, FixedI16, FixedU16};

/// System reference for a record (Table 9.4.1.5-1, bits 3..2).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reference {
    /// Not specified, or set in the global header word.
    #[default]
    NotSpecified,
    /// Earth-Centered Earth-Fixed.
    Ecef,
    /// Platform centered.
    PlatformCentered,
    /// Array centered.
    ArrayCentered,
}

impl Reference {
    fn from_code(code: u32) -> Reference {
        match code & 0b11 {
            0 => Reference::NotSpecified,
            1 => Reference::Ecef,
            2 => Reference::PlatformCentered,
            _ => Reference::ArrayCentered,
        }
    }
    fn code(self) -> u32 {
        match self {
            Reference::NotSpecified => 0,
            Reference::Ecef => 1,
            Reference::PlatformCentered => 2,
            Reference::ArrayCentered => 3,
        }
    }
}

/// Whether a record's pointing vector is a beam or a null (Table 9.4.1.5-2, bits 1..0).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Beam {
    /// Not specified, or set in the global header word.
    #[default]
    NotSpecified,
    /// Beam, or signal direction.
    Beam,
    /// Null.
    Null,
    /// Reserved.
    Reserved,
}

impl Beam {
    fn from_code(code: u32) -> Beam {
        match code & 0b11 {
            0 => Beam::NotSpecified,
            1 => Beam::Beam,
            2 => Beam::Null,
            _ => Beam::Reserved,
        }
    }
    fn code(self) -> u32 {
        match self {
            Beam::NotSpecified => 0,
            Beam::Beam => 1,
            Beam::Null => 2,
            Beam::Reserved => 3,
        }
    }
}

/// The Index/Reference/Beam subfield word (§9.4.1.5): a 16-bit Record Index
/// (bits 31..16), a 2-bit Reference (bits 3..2), and a 2-bit Beam (bits 1..0).
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexRefBeam(u32);

impl IndexRefBeam {
    /// Build an Index/Reference/Beam word. When used as the global header word the
    /// index must be 0 (Rule 9.4.1.5-2).
    pub fn new(index: u16, reference: Reference, beam: Beam) -> IndexRefBeam {
        IndexRefBeam((u32::from(index) << 16) | (reference.code() << 2) | beam.code())
    }
    /// The 16-bit Record Index (0 = unused, Rules 9.4.1.5-4/-5).
    pub fn index(&self) -> u16 {
        (self.0 >> 16) as u16
    }
    /// The system Reference (Table 9.4.1.5-1).
    pub fn reference(&self) -> Reference {
        Reference::from_code(self.0 >> 2)
    }
    /// The Beam/Null indication (Table 9.4.1.5-2).
    pub fn beam(&self) -> Beam {
        Beam::from_code(self.0)
    }
}

/// The single-word 3-D Pointing Vector (§9.4.1.1): a signed Elevation angle in
/// bits 31..16 and an unsigned Azimuthal angle in bits 15..0, both in degrees with
/// a radix point at bit 7 of their subfield (1/128° resolution).
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointingVector(u32);

impl PointingVector {
    /// Build a pointing vector from elevation (`[-90, 90]`°) and azimuth
    /// (`[0, 511.9921875]`°) in degrees.
    pub fn new(elevation_deg: f32, azimuth_deg: f32) -> PointingVector {
        let elev = FixedI16::<U7>::from_num(elevation_deg).to_bits();
        let azim = FixedU16::<U7>::from_num(azimuth_deg).to_bits();
        PointingVector((u32::from(elev as u16) << 16) | u32::from(azim))
    }
    /// Elevation angle in degrees (signed, Rule 9.4.1.1-3).
    pub fn elevation_deg(&self) -> f32 {
        FixedI16::<U7>::from_bits((self.0 >> 16) as i16).to_num()
    }
    /// Azimuthal angle in degrees (unsigned, Rule 9.4.1.1-2).
    pub fn azimuth_deg(&self) -> f32 {
        FixedU16::<U7>::from_bits(self.0 as u16).to_num()
    }
}

/// One record of a 3-D Pointing Vector Structure: the required Pointing Vector plus
/// the optional Index/Reference/Beam word. Presence of each is fixed for the whole
/// field by the Control/Context Indicator word (§9.4.1.4).
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian, cif: u32")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointingRecord {
    /// Index/Reference/Beam word — present iff CIF bit 31 (Rule 9.4.1.4-1).
    #[deku(cond = "cif & (1 << 31) != 0")]
    index_ref_beam: Option<IndexRefBeam>,
    /// The pointing vector — present iff CIF bit 30, which is always set for a
    /// well-formed field (Observation 9.4.1.4-1).
    #[deku(cond = "cif & (1 << 30) != 0")]
    pointing: Option<PointingVector>,
}

impl PointingRecord {
    /// A record with just a pointing vector (no per-record Index/Reference/Beam).
    pub fn new(pointing: PointingVector) -> PointingRecord {
        PointingRecord {
            index_ref_beam: None,
            pointing: Some(pointing),
        }
    }
    /// A record with a pointing vector and an Index/Reference/Beam word.
    pub fn with_index_ref_beam(pointing: PointingVector, irb: IndexRefBeam) -> PointingRecord {
        PointingRecord {
            index_ref_beam: Some(irb),
            pointing: Some(pointing),
        }
    }
    /// The record's pointing vector.
    pub fn pointing(&self) -> Option<PointingVector> {
        self.pointing
    }
    /// The record's Index/Reference/Beam word, if present.
    pub fn index_ref_beam(&self) -> Option<IndexRefBeam> {
        self.index_ref_beam
    }
}

/// The 3-D Pointing Vector Structure (§9.4.1.2): an Array-of-Records with a
/// three-word header (size, record geometry, and the Control/Context Indicator),
/// an optional global Index/Reference/Beam word, and the records.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThreeDPointingVectorStruct {
    /// Total field size in 32-bit words (word 1).
    size_of_array: u32,
    /// HeaderSize [31:24] | # words/record [23:12] | # records [11:0] (word 2).
    header_word: u32,
    /// Control/Context Indicator: bit 31 = Index/Ref/Beam present, bit 30 =
    /// Pointing Vector present (word 3, §9.4.1.4).
    cif: u32,
    /// Optional global Index/Reference/Beam word — present iff HeaderSize == 4
    /// (Rule 9.4.1.3-1/-2).
    #[deku(cond = "(header_word >> 24) & 0xFF == 4")]
    global_index_ref_beam: Option<IndexRefBeam>,
    #[deku(count = "(header_word & 0xFFF) as usize", ctx = "*cif")]
    records: Vec<PointingRecord>,
}

impl ThreeDPointingVectorStruct {
    /// Build a 3-D Pointing Vector Structure from its records and an optional global
    /// Index/Reference/Beam word. The header words (size, geometry, CIF) are derived
    /// from the records; whether the per-record Index/Reference/Beam word is present
    /// is taken from the first record and must be uniform across records
    /// (§9.4.1.2 — every record's structure must be identical).
    pub fn new(
        records: Vec<PointingRecord>,
        global: Option<IndexRefBeam>,
    ) -> ThreeDPointingVectorStruct {
        let has_irb = records.first().is_some_and(|r| r.index_ref_beam.is_some());
        // CIF bit 30 (pointing) is always set; bit 31 iff records carry Index/Ref/Beam.
        let cif = (1 << 30) | if has_irb { 1 << 31 } else { 0 };
        let words_per_rec = 1 + u32::from(has_irb);
        let header_size = 3 + u32::from(global.is_some());
        let num_records = records.len() as u32;
        ThreeDPointingVectorStruct {
            size_of_array: header_size + words_per_rec * num_records,
            header_word: (header_size << 24)
                | ((words_per_rec & 0xFFF) << 12)
                | (num_records & 0xFFF),
            cif,
            global_index_ref_beam: global,
            records,
        }
    }

    /// The records of the structure.
    pub fn records(&self) -> &[PointingRecord] {
        &self.records
    }

    /// The global Index/Reference/Beam word, if present.
    pub fn global_index_ref_beam(&self) -> Option<IndexRefBeam> {
        self.global_index_ref_beam
    }

    /// Size of the whole field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        self.size_of_array as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn pointing_vector_radix_roundtrip() {
        let pv = PointingVector::new(-45.5, 271.0);
        assert!((pv.elevation_deg() - -45.5).abs() < 1.0 / 128.0);
        assert!((pv.azimuth_deg() - 271.0).abs() < 1.0 / 128.0);
        // Negative elevation must not corrupt the azimuth half-word.
        assert!((pv.azimuth_deg() - 271.0).abs() < 1.0 / 128.0);
    }

    #[test]
    fn structure_roundtrips_through_a_context_packet() {
        let records = vec![
            PointingRecord::with_index_ref_beam(
                PointingVector::new(10.0, 20.0),
                IndexRefBeam::new(1, Reference::Ecef, Beam::Beam),
            ),
            PointingRecord::with_index_ref_beam(
                PointingVector::new(-30.0, 350.5),
                IndexRefBeam::new(2, Reference::PlatformCentered, Beam::Null),
            ),
        ];
        let field = ThreeDPointingVectorStruct::new(
            records,
            Some(IndexRefBeam::new(0, Reference::Ecef, Beam::NotSpecified)),
        );
        // header 4 (global) + 2 words/record * 2 records = 8.
        assert_eq!(field.size_words(), 8);

        let mut packet = Vrt::new_context_packet();
        packet
            .payload_mut()
            .context_mut()
            .unwrap()
            .set_three_d_pointing_vector_struct(Some(field.clone()));
        packet.update_packet_size();

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        let got = parsed
            .payload()
            .context()
            .unwrap()
            .three_d_pointing_vector_struct()
            .unwrap()
            .clone();
        assert_eq!(got, field);
        assert_eq!(got.records().len(), 2);
        assert_eq!(
            got.records()[0].index_ref_beam().unwrap().reference(),
            Reference::Ecef
        );
        assert!((got.records()[1].pointing().unwrap().azimuth_deg() - 350.5).abs() < 1.0 / 128.0);
    }
}
