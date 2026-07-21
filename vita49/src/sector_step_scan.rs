// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
The Sector/Step-Scan field (ANSI/VITA-49.2-2017 §9.6.2, CIF1 bit 9): an
Array-of-Records field setting up a multi-sectored scanning/stepping receiver.
*/

use crate::gain::Gain;
use crate::packet_header::PrologueCtx;
use crate::state_time::StateTime;
use crate::threshold::Threshold;
use deku::prelude::*;
use fixed::{types::extra::U20, FixedU64};

fn hz_to_bits(hz: f64) -> u64 {
    FixedU64::<U20>::from_num(hz).to_bits()
}
fn bits_to_hz(bits: u64) -> f64 {
    FixedU64::<U20>::from_bits(bits).to_num()
}

/// One record of a Sector/Step-Scan field (§9.6.2.2..§9.6.2.13). Two subfields are
/// required (Sector Number, F1 Start Frequency); the rest are optional. Which are
/// present is fixed for the whole field by the Control/Context Indicator word.
///
/// The frequency subfields are 64-bit radix-20 Hz (Rules 9.6.2.3..-6). The Dwell,
/// Time 3 and Time 4 subfields are 64-bit Fractional Time in femtoseconds (Rules
/// 9.6.2.10-2/12-1/13-1) — NB the Table 9.6.2.1-1 word counts for these are a
/// documentation error (they are 2 words, not 1). Start Time is prologue-TSI/TSF
/// sized (Rule 9.6.2.11-1).
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, cif: u32, prologue: PrologueCtx"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SectorRecord {
    #[deku(cond = "cif & (1 << 31) != 0")]
    sector_number: Option<u32>,
    #[deku(cond = "cif & (1 << 30) != 0")]
    f1_start_freq: Option<u64>,
    #[deku(cond = "cif & (1 << 29) != 0")]
    f2_stop_freq: Option<u64>,
    #[deku(cond = "cif & (1 << 28) != 0")]
    resolution_bw: Option<u64>,
    #[deku(cond = "cif & (1 << 27) != 0")]
    tune_step: Option<u64>,
    #[deku(cond = "cif & (1 << 26) != 0")]
    num_points: Option<u32>,
    #[deku(cond = "cif & (1 << 25) != 0")]
    default_gain: Option<Gain>,
    #[deku(cond = "cif & (1 << 24) != 0")]
    threshold: Option<Threshold>,
    #[deku(cond = "cif & (1 << 23) != 0")]
    dwell_time: Option<i64>,
    #[deku(cond = "cif & (1 << 22) != 0", ctx = "prologue")]
    start_time: Option<StateTime>,
    #[deku(cond = "cif & (1 << 21) != 0")]
    time3: Option<i64>,
    #[deku(cond = "cif & (1 << 20) != 0")]
    time4: Option<i64>,
}

impl SectorRecord {
    /// A record with the two required subfields: sector number and F1 start
    /// frequency (Hz). Add optional subfields with the `with_*` methods.
    pub fn new(sector_number: u32, f1_start_freq_hz: f64) -> SectorRecord {
        SectorRecord {
            sector_number: Some(sector_number),
            f1_start_freq: Some(hz_to_bits(f1_start_freq_hz)),
            ..Default::default()
        }
    }

    /// Set the F2 stop frequency (Hz) — presence indicates scanning (vs stepping).
    pub fn with_f2_stop_freq_hz(mut self, hz: f64) -> SectorRecord {
        self.f2_stop_freq = Some(hz_to_bits(hz));
        self
    }
    /// Set the resolution bandwidth (Hz).
    pub fn with_resolution_bw_hz(mut self, hz: f64) -> SectorRecord {
        self.resolution_bw = Some(hz_to_bits(hz));
        self
    }
    /// Set the tune step size (Hz).
    pub fn with_tune_step_hz(mut self, hz: f64) -> SectorRecord {
        self.tune_step = Some(hz_to_bits(hz));
        self
    }
    /// Set the number of points.
    pub fn with_num_points(mut self, n: u32) -> SectorRecord {
        self.num_points = Some(n);
        self
    }
    /// Set the default gain.
    pub fn with_default_gain(mut self, gain: Gain) -> SectorRecord {
        self.default_gain = Some(gain);
        self
    }
    /// Set the threshold.
    pub fn with_threshold(mut self, threshold: Threshold) -> SectorRecord {
        self.threshold = Some(threshold);
        self
    }
    /// Set the dwell time (femtoseconds, §9.7 Fractional Time).
    pub fn with_dwell_time_fs(mut self, fs: i64) -> SectorRecord {
        self.dwell_time = Some(fs);
        self
    }
    /// Set the start time. Its present integer/fractional parts must match the
    /// packet prologue TSI/TSF (see [`StateTime`]).
    pub fn with_start_time(mut self, start: StateTime) -> SectorRecord {
        self.start_time = Some(start);
        self
    }
    /// Set the Time 3 ("pre-dwell") subfield (femtoseconds).
    pub fn with_time3_fs(mut self, fs: i64) -> SectorRecord {
        self.time3 = Some(fs);
        self
    }
    /// Set the Time 4 ("loss-dwell") subfield (femtoseconds).
    pub fn with_time4_fs(mut self, fs: i64) -> SectorRecord {
        self.time4 = Some(fs);
        self
    }

    /// Sector number (required).
    pub fn sector_number(&self) -> Option<u32> {
        self.sector_number
    }
    /// F1 start frequency in Hz (required).
    pub fn f1_start_freq_hz(&self) -> Option<f64> {
        self.f1_start_freq.map(bits_to_hz)
    }
    /// F2 stop frequency in Hz.
    pub fn f2_stop_freq_hz(&self) -> Option<f64> {
        self.f2_stop_freq.map(bits_to_hz)
    }
    /// Resolution bandwidth in Hz.
    pub fn resolution_bw_hz(&self) -> Option<f64> {
        self.resolution_bw.map(bits_to_hz)
    }
    /// Tune step size in Hz.
    pub fn tune_step_hz(&self) -> Option<f64> {
        self.tune_step.map(bits_to_hz)
    }
    /// Number of points.
    pub fn num_points(&self) -> Option<u32> {
        self.num_points
    }
    /// Default gain.
    pub fn default_gain(&self) -> Option<Gain> {
        self.default_gain
    }
    /// Threshold.
    pub fn threshold(&self) -> Option<Threshold> {
        self.threshold
    }
    /// Dwell time in femtoseconds.
    pub fn dwell_time_fs(&self) -> Option<i64> {
        self.dwell_time
    }
    /// Start time.
    pub fn start_time(&self) -> Option<StateTime> {
        self.start_time
    }
    /// Time 3 in femtoseconds.
    pub fn time3_fs(&self) -> Option<i64> {
        self.time3
    }
    /// Time 4 in femtoseconds.
    pub fn time4_fs(&self) -> Option<i64> {
        self.time4
    }

    /// The Control/Context Indicator bits this record's present subfields set.
    fn cif_bits(&self) -> u32 {
        (u32::from(self.sector_number.is_some()) << 31)
            | (u32::from(self.f1_start_freq.is_some()) << 30)
            | (u32::from(self.f2_stop_freq.is_some()) << 29)
            | (u32::from(self.resolution_bw.is_some()) << 28)
            | (u32::from(self.tune_step.is_some()) << 27)
            | (u32::from(self.num_points.is_some()) << 26)
            | (u32::from(self.default_gain.is_some()) << 25)
            | (u32::from(self.threshold.is_some()) << 24)
            | (u32::from(self.dwell_time.is_some()) << 23)
            | (u32::from(self.start_time.is_some()) << 22)
            | (u32::from(self.time3.is_some()) << 21)
            | (u32::from(self.time4.is_some()) << 20)
    }

    /// This record's size in 32-bit words (the sum of its present subfields).
    fn word_count(&self) -> u32 {
        u32::from(self.sector_number.is_some())
            + 2 * u32::from(self.f1_start_freq.is_some())
            + 2 * u32::from(self.f2_stop_freq.is_some())
            + 2 * u32::from(self.resolution_bw.is_some())
            + 2 * u32::from(self.tune_step.is_some())
            + u32::from(self.num_points.is_some())
            + u32::from(self.default_gain.is_some())
            + u32::from(self.threshold.is_some())
            + 2 * u32::from(self.dwell_time.is_some())
            + self.start_time.map_or(0, |s| u32::from(s.size_words()))
            + 2 * u32::from(self.time3.is_some())
            + 2 * u32::from(self.time4.is_some())
    }
}

/// The Sector/Step-Scan field (§9.6.2): a three-word Array-of-Records header
/// (size, record geometry, Control/Context Indicator) followed by the records.
/// Per §9.6.2.1 every record's structure must be identical.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, prologue: PrologueCtx"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SectorStepScan {
    size_of_array: u32,
    header_word: u32,
    cif: u32,
    #[deku(count = "(header_word & 0xFFF) as usize", ctx = "*cif, prologue")]
    records: Vec<SectorRecord>,
}

impl SectorStepScan {
    /// Build a Sector/Step-Scan field from its records. The header (size, geometry,
    /// Control/Context Indicator) is derived from the first record; every record
    /// must have the same structure (§9.6.2.1).
    pub fn new(records: Vec<SectorRecord>) -> SectorStepScan {
        let (cif, words_per_rec) = records
            .first()
            .map_or((0, 0), |r| (r.cif_bits(), r.word_count()));
        let num_records = records.len() as u32;
        SectorStepScan {
            size_of_array: 3 + words_per_rec * num_records,
            header_word: (3 << 24) | ((words_per_rec & 0xFFF) << 12) | (num_records & 0xFFF),
            cif,
            records,
        }
    }

    /// The records of the field.
    pub fn records(&self) -> &[SectorRecord] {
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
    fn sector_step_scan_round_trips_with_prologue_start_time() {
        let mut packet = Vrt::new_context_packet();
        // Prologue with both TSI and TSF, so Start Time is 3 words.
        packet.set_integer_timestamp(Some(1), Tsi::Utc).unwrap();
        packet
            .set_fractional_timestamp(Some(2), Tsf::RealTimePs)
            .unwrap();

        let rec = SectorRecord::new(1, 100e6)
            .with_f2_stop_freq_hz(200e6)
            .with_resolution_bw_hz(1e3)
            .with_default_gain(Gain::new(10.0, -3.0))
            .with_dwell_time_fs(5_000_000)
            .with_start_time(StateTime::new(Some(9), Some(-42)))
            .with_time3_fs(1_000);
        let field = SectorStepScan::new(vec![rec, rec]);
        // header 3 + words/rec (1 + 2 + 2 + 2 + 1 + 2 + 3 + 2 = 15) * 2 records = 33.
        assert_eq!(field.size_words(), 33);

        packet
            .payload_mut()
            .context_mut()
            .unwrap()
            .set_sector_scan(Some(field.clone()));
        packet.update_packet_size();

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        let got = parsed.payload().context().unwrap().sector_scan().unwrap();
        assert_eq!(got, &field);
        let r0 = &got.records()[0];
        assert_eq!(r0.sector_number(), Some(1));
        assert_eq!(r0.f2_stop_freq_hz(), Some(200e6));
        assert_eq!(r0.start_time(), Some(StateTime::new(Some(9), Some(-42))));
        assert_eq!(r0.time3_fs(), Some(1_000));
    }
}
