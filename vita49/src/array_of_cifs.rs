// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
The Array of CIF Fields (ANSI/VITA-49.2-2017 §9.13.1, CIF1 bit 11): an
Array-of-Records field whose records each carry a selection of the CIF0..CIF3
fields, chosen by four global selector words.
*/

use std::cell::Cell;

use crate::cif0::{Cif0, Cif0Fields};
use crate::cif1::{Cif1, Cif1Fields};
use crate::cif2::{Cif2, Cif2Fields};
use crate::cif3::{Cif3, Cif3Fields};
use crate::cif7::Cif7Opts;
use crate::packet_header::PrologueCtx;
use deku::prelude::*;

// A record can itself select CIF1 bit 11 (this field), so parsing recurses. Bound
// the nesting depth so a crafted packet can't exhaust the stack. The counter is
// per-thread and decremented on every exit (including errors) by the scope guard.
thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}
const MAX_DEPTH: u32 = 8;

struct DepthGuard;
impl Drop for DepthGuard {
    fn drop(&mut self) {
        DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// One record of an Array of CIF Fields (§9.13.1): the array index followed by the
/// selected fields of CIF0, CIF1, CIF2 and CIF3 (in that order; Rule 9.13.1-2/-4).
/// The four global selector words determine which fields each record carries.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, cif0: Cif0, cif1: Cif1, cif2: Cif2, cif3: Cif3, prologue: PrologueCtx"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CifRecord {
    index: u32,
    #[deku(ctx = "&cif0, Cif7Opts::default(), prologue")]
    cif0_fields: Cif0Fields,
    #[deku(ctx = "Some(&cif1), Cif7Opts::default(), prologue")]
    cif1_fields: Cif1Fields,
    #[deku(ctx = "Some(&cif2), Cif7Opts::default(), prologue")]
    cif2_fields: Cif2Fields,
    #[deku(ctx = "Some(&cif3), Cif7Opts::default(), prologue")]
    cif3_fields: Cif3Fields,
}

impl CifRecord {
    /// A record carrying the given array index and per-CIF field sets.
    pub fn new(
        index: u32,
        cif0_fields: Cif0Fields,
        cif1_fields: Cif1Fields,
        cif2_fields: Cif2Fields,
        cif3_fields: Cif3Fields,
    ) -> CifRecord {
        CifRecord {
            index,
            cif0_fields,
            cif1_fields,
            cif2_fields,
            cif3_fields,
        }
    }

    /// The array index of this record.
    pub fn index(&self) -> u32 {
        self.index
    }
    /// The record's CIF0 fields.
    pub fn cif0_fields(&self) -> &Cif0Fields {
        &self.cif0_fields
    }
    /// The record's CIF1 fields.
    pub fn cif1_fields(&self) -> &Cif1Fields {
        &self.cif1_fields
    }
    /// The record's CIF2 fields.
    pub fn cif2_fields(&self) -> &Cif2Fields {
        &self.cif2_fields
    }
    /// The record's CIF3 fields.
    pub fn cif3_fields(&self) -> &Cif3Fields {
        &self.cif3_fields
    }

    fn word_count(&self) -> u32 {
        1 + u32::from(self.cif0_fields.size_words())
            + u32::from(self.cif1_fields.size_words())
            + u32::from(self.cif2_fields.size_words())
            + u32::from(self.cif3_fields.size_words())
    }
}

/// The Array of CIF Fields (§9.13.1): a three-word Array-of-Records header
/// (`HeaderSize` = 7, Rule 9.13.1-3) plus four global CIF0..CIF3 selector words,
/// then the records. Per Rule 9.13.1-3 the header is 7 words (3 mandatory + 4 CIF
/// selectors); the CIF7 selector shown in Figure 9.13.1-1 is not present (the
/// figure and Rule 9.13.1-3 disagree; the Rule is authoritative).
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayOfCifs {
    size_of_array: u32,
    header_word: u32,
    // Bitmapped Control/Context Indicator — unused, set to 0 (Rule text).
    cc_indicator: u32,
    cif0_selector: Cif0,
    cif1_selector: Cif1,
    cif2_selector: Cif2,
    cif3_selector: Cif3,
    records: Vec<CifRecord>,
}

// Both directions are hand-written: the reader wraps the recursion-depth guard,
// and the writer must emit the records the derive would otherwise need matching
// `count`/`ctx` attributes for.
impl DekuWriter<(deku::ctx::Endian, PrologueCtx)> for ArrayOfCifs {
    fn to_writer<W: deku::no_std_io::Write + deku::no_std_io::Seek>(
        &self,
        writer: &mut deku::writer::Writer<W>,
        (endian, prologue): (deku::ctx::Endian, PrologueCtx),
    ) -> Result<(), DekuError> {
        self.size_of_array.to_writer(writer, endian)?;
        self.header_word.to_writer(writer, endian)?;
        self.cc_indicator.to_writer(writer, endian)?;
        self.cif0_selector.to_writer(writer, endian)?;
        self.cif1_selector.to_writer(writer, endian)?;
        self.cif2_selector.to_writer(writer, endian)?;
        self.cif3_selector.to_writer(writer, endian)?;
        for r in &self.records {
            r.to_writer(
                writer,
                (
                    endian,
                    self.cif0_selector,
                    self.cif1_selector,
                    self.cif2_selector,
                    self.cif3_selector,
                    prologue,
                ),
            )?;
        }
        Ok(())
    }
}

// Read is hand-written so the recursion-depth guard can wrap it; the derived
// writer above just emits the fields (writing a finite structure can't recurse).
impl<'a> DekuReader<'a, (deku::ctx::Endian, PrologueCtx)> for ArrayOfCifs {
    fn from_reader_with_ctx<R: deku::no_std_io::Read + deku::no_std_io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        (endian, prologue): (deku::ctx::Endian, PrologueCtx),
    ) -> Result<ArrayOfCifs, DekuError> {
        let depth = DEPTH.with(|d| {
            let v = d.get() + 1;
            d.set(v);
            v
        });
        let _guard = DepthGuard;
        if depth > MAX_DEPTH {
            return Err(DekuError::Parse(
                "Array of CIF Fields nested beyond the supported depth",
            ));
        }

        let size_of_array = u32::from_reader_with_ctx(reader, endian)?;
        let header_word = u32::from_reader_with_ctx(reader, endian)?;
        let cc_indicator = u32::from_reader_with_ctx(reader, endian)?;
        let cif0_selector = Cif0::from_reader_with_ctx(reader, endian)?;
        let cif1_selector = Cif1::from_reader_with_ctx(reader, endian)?;
        let cif2_selector = Cif2::from_reader_with_ctx(reader, endian)?;
        let cif3_selector = Cif3::from_reader_with_ctx(reader, endian)?;

        let num_records = (header_word & 0xFFF) as usize;
        let mut records = Vec::new();
        for _ in 0..num_records {
            records.push(CifRecord::from_reader_with_ctx(
                reader,
                (
                    endian,
                    cif0_selector,
                    cif1_selector,
                    cif2_selector,
                    cif3_selector,
                    prologue,
                ),
            )?);
        }

        Ok(ArrayOfCifs {
            size_of_array,
            header_word,
            cc_indicator,
            cif0_selector,
            cif1_selector,
            cif2_selector,
            cif3_selector,
            records,
        })
    }
}

impl ArrayOfCifs {
    /// Build an Array of CIF Fields from the four CIF selectors and the records.
    /// The caller supplies selectors matching the fields set in each record.
    pub fn new(
        cif0_selector: Cif0,
        cif1_selector: Cif1,
        cif2_selector: Cif2,
        cif3_selector: Cif3,
        records: Vec<CifRecord>,
    ) -> ArrayOfCifs {
        let words_per_rec = records.first().map_or(0, CifRecord::word_count);
        let num_records = records.len() as u32;
        ArrayOfCifs {
            size_of_array: 7 + words_per_rec * num_records,
            header_word: (7 << 24) | ((words_per_rec & 0xFFF) << 12) | (num_records & 0xFFF),
            cc_indicator: 0,
            cif0_selector,
            cif1_selector,
            cif2_selector,
            cif3_selector,
            records,
        }
    }

    /// The records of the array.
    pub fn records(&self) -> &[CifRecord] {
        &self.records
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
    fn array_of_cifs_round_trips_through_a_context_packet() {
        // Records with no CIF fields selected (empty selectors) — each record is
        // just its index word.
        let records = vec![
            CifRecord::new(
                0,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ),
            CifRecord::new(
                1,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ),
        ];
        let field = ArrayOfCifs::new(
            Cif0::default(),
            Cif1::default(),
            Cif2::default(),
            Cif3::default(),
            records,
        );
        // 7 header + 1 word/record (index only) * 2 records = 9.
        assert_eq!(field.size_words(), 9);

        let mut packet = Vrt::new_context_packet();
        packet
            .payload_mut()
            .context_mut()
            .unwrap()
            .set_array_of_cifs(Some(field.clone()));
        packet.update_packet_size();

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        let got = parsed.payload().context().unwrap().array_of_cifs().unwrap();
        assert_eq!(got, &field);
        assert_eq!(got.records().len(), 2);
        assert_eq!(got.records()[1].index(), 1);
    }

    #[test]
    fn deeply_nested_array_of_cifs_is_rejected_not_a_stack_overflow() {
        // Each level's record selects CIF1 bit 11 (this field), nesting an inner
        // Array of CIF Fields. Parsing past MAX_DEPTH must return an error, not
        // recurse until the stack overflows.
        let cif1_sel = {
            let mut c = Cif1::default();
            c.set_array_of_cifs();
            c
        };
        let mut inner = ArrayOfCifs::new(
            Cif0::default(),
            Cif1::default(),
            Cif2::default(),
            Cif3::default(),
            vec![CifRecord::new(
                0,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            )],
        );
        for _ in 0..(MAX_DEPTH + 2) {
            let record = CifRecord::new(
                0,
                Default::default(),
                Cif1Fields {
                    array_of_cifs: Some(inner),
                    ..Default::default()
                },
                Default::default(),
                Default::default(),
            );
            inner = ArrayOfCifs::new(
                Cif0::default(),
                cif1_sel,
                Cif2::default(),
                Cif3::default(),
                vec![record],
            );
        }

        let mut packet = Vrt::new_context_packet();
        packet
            .payload_mut()
            .context_mut()
            .unwrap()
            .set_array_of_cifs(Some(inner));
        packet.update_packet_size();

        assert!(Vrt::try_from(packet.to_bytes().unwrap().as_ref()).is_err());
    }
}
