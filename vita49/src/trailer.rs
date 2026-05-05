// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the trailer field
(ANSI/VITA-49.2-2017 section 5.1.6).

The trailer is an optional field whose presence is identified by the `T` bit in
the packet header. It carries state and event indicators that describe the
validity of the signal data and the status of the processes that produced it.
It also optionally carries a count of associated Context packets.

See VITA 49.2 §5.1.6, Figure 5.1.6-1 for the layout of the 32-bit trailer word.

Each of the eight predefined indicator bits (bits 19–12) has a corresponding
enable bit (bits 31–24) in the same relative position within the field. An
indicator is only considered valid when its enable bit is set to one.
*/

use deku::prelude::*;

/// Sample frame indicator enumeration.
///
/// Encodes the position of this packet within the current Sample Frame.
/// The corresponding enable bits are bits 23 and 22 of the trailer word; the
/// indicator value itself occupies bits 11–10. Both enable bits must be set
/// for the indicator to be considered valid (VITA 49.2 §5.1.6, Table 5.1.6-1,
/// user-defined indicators [23..20] / [11..8]).
/// See VITA 49.2 §3.4 & §5.1.6.1 for details.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SampleFrameIndicator {
    /// Sample Frames are not applicable to data packets, or the entire Sample
    /// Frame is contained in a single data packet
    #[deku(id = 0x0)]
    NotApplicable = 0,
    /// First data packet of current Sample Frame
    #[deku(id = 0x1)]
    FirstDataPacket = 1,
    /// Middle packet or packets of Sample Frame, i.e. "continuation" indicator
    #[deku(id = 0x2)]
    MiddleDataPacket = 2,
    /// Final data packet of current Sample Frame
    #[deku(id = 0x3)]
    FinalDataPacket = 3,
}

impl TryFrom<u32> for SampleFrameIndicator {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotApplicable),
            1 => Ok(Self::FirstDataPacket),
            2 => Ok(Self::MiddleDataPacket),
            3 => Ok(Self::FinalDataPacket),
            _ => Err(()),
        }
    }
}

/// Base trailer field data structure.
///
/// The trailer is an optional 32-bit word appended to Signal Data packets when
/// the `T` bit in the packet header is set. It is structured as three logical
/// sections (VITA 49.2 §5.1.6):
///
/// 1. **Enables** (bits 31–20): one enable bit per predefined indicator. An
///    indicator is only valid when its enable bit is `1`.
/// 2. **State and Event Indicators** (bits 19–8): one indicator bit per
///    predefined field, plus four user-defined indicator bits.
/// 3. **Associated Context Packet Count** (bits 7–0): an optional 7-bit count
///    (bit 7 is the enable `E` flag) of Context packets associated with this
///    Data packet.
///
/// | Field                       | Enable bit(s) | Indicator bit(s) |
/// |-----------------------------|:-------------:|:----------------:|
/// | Calibrated Time             | 31            | 19               |
/// | Valid Data                  | 30            | 18               |
/// | Reference Lock              | 29            | 17               |
/// | AGC/MGC                     | 28            | 16               |
/// | Detected Signal             | 27            | 15               |
/// | Spectral Inversion          | 26            | 14               |
/// | Over-range                  | 25            | 13               |
/// | Sample Loss                 | 24            | 12               |
/// | Sample Frame                | 23–22         | 11–10            |
/// | User-Defined                | 21–20         | 9–8              |
/// | Assoc. Context Packet Count | 7 (`E`)       | 6–0              |
///
/// ## Additional Notes
/// * Enable bits 31–24 / indicator bits 19–12: one enable bit gates one boolean
///   indicator bit.
/// * Enable bits 23–20 and bit 7 / indicator bits 11–8 and 6–0:
///   all enable bits in the range must be set for the field to be considered
///   valid; the indicator bits then encode a numeric value rather than a boolean.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trailer(u32);

impl Trailer {
    /// Construct a zero'd Trailer word
    ///
    /// Synonymous with `Default` implementation.
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets the raw 32-bit value of the trailer.
    pub fn as_u32(self) -> u32 {
        self.0
    }
    /// Returns `true` if the Calibrated Time enable bit (bit 31) is set,
    /// meaning [`cal_time_indicator`](Self::cal_time_indicator) carries a valid
    /// value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn cal_time_enabled(&self) -> bool {
        self.0 & (1 << 31) > 0
    }
    /// Returns `true` if the Valid Data enable bit (bit 30) is set, meaning
    /// [`valid_data_indicator`](Self::valid_data_indicator) carries a valid
    /// value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn valid_data_enabled(&self) -> bool {
        self.0 & (1 << 30) > 0
    }
    /// Returns `true` if the Reference Lock enable bit (bit 29) is set,
    /// meaning [`reference_lock_indicator`](Self::reference_lock_indicator)
    /// carries a valid value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn reference_lock_enabled(&self) -> bool {
        self.0 & (1 << 29) > 0
    }
    /// Returns `true` if the AGC/MGC enable bit (bit 28) is set, meaning
    /// [`agc_indicator`](Self::agc_indicator) carries a valid value
    /// (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn agc_enabled(&self) -> bool {
        self.0 & (1 << 28) > 0
    }
    /// Returns `true` if the Detected Signal enable bit (bit 27) is set,
    /// meaning [`detected_signal_indicator`](Self::detected_signal_indicator)
    /// carries a valid value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn detected_signal_enabled(&self) -> bool {
        self.0 & (1 << 27) > 0
    }
    /// Returns `true` if the Spectral Inversion enable bit (bit 26) is set,
    /// meaning
    /// [`spectral_inversion_indicator`](Self::spectral_inversion_indicator)
    /// carries a valid value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn spectral_inversion_enabled(&self) -> bool {
        self.0 & (1 << 26) > 0
    }
    /// Returns `true` if the Over-range enable bit (bit 25) is set, meaning
    /// [`over_range_indicator`](Self::over_range_indicator) carries a valid
    /// value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn over_range_enabled(&self) -> bool {
        self.0 & (1 << 25) > 0
    }
    /// Returns `true` if the Sample Loss enable bit (bit 24) is set, meaning
    /// [`sample_loss_indicator`](Self::sample_loss_indicator) carries a valid
    /// value (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn sample_loss_enabled(&self) -> bool {
        self.0 & (1 << 24) > 0
    }
    /// Returns `true` if both Sample Frame enable bits (bits 23 and 22) are
    /// set, meaning [`sample_frame_indicator`](Self::sample_frame_indicator)
    /// carries a valid value. Both bits must be `1` per the user-defined
    /// multi-bit enable rule (VITA 49.2 §5.1.6, Rule 5.1.6-4).
    pub fn sample_frame_enabled(&self) -> bool {
        self.0 & (1 << 23) > 0 && self.0 & (1 << 22) > 0
    }
    /// Returns `true` if both user-defined enable bits (bits 21 and 20) are
    /// set, meaning [`user_defined_indicator`](Self::user_defined_indicator)
    /// carries a valid value. Both bits must be `1` per the user-defined
    /// multi-bit enable rule (VITA 49.2 §5.1.6, Rule 5.1.6-4).
    pub fn user_defined_enabled(&self) -> bool {
        self.0 & (1 << 21) > 0 && self.0 & (1 << 20) > 0
    }
    /// Returns `true` if the Associated Context Packet Count enable bit (`E`,
    /// bit 7) is set, meaning
    /// [`associated_context_packet_count`](Self::associated_context_packet_count)
    /// carries a valid count (VITA 49.2 §5.1.6, Rule 5.1.6-13).
    pub fn associated_context_packet_count_enabled(&self) -> bool {
        self.0 & (1 << 7) > 0
    }
    /// Returns the calibration time indicator status if the enable bit (bit 31)
    /// is set.
    ///
    /// When `Some(true)`, the Timestamp in the packet is calibrated to an
    /// external reference. When `Some(false)`, the Timestamp is free-running
    /// and may be inaccurate. Returns `None` when the enable bit is clear and
    /// the indicator value is undefined (VITA 49.2 §5.1.6, Rule 5.1.6-5).
    pub fn cal_time_indicator(&self) -> Option<bool> {
        self.cal_time_enabled().then_some(self.0 & (1 << 19) > 0)
    }
    /// Sets the calibration time indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 31) and writes `v` to bit 19.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_cal_time_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 31) & !(1 << 19);
        if let Some(v) = value {
            self.0 |= 1 << 31;
            self.0 |= (v as u32) << 19;
        }
    }
    /// Returns the valid data indicator status if the enable bit (bit 30) is
    /// set.
    ///
    /// When `Some(true)`, the payload data in the packet is valid. When
    /// `Some(false)`, some condition exists that may invalidate the data (e.g.,
    /// a frequency change is in progress). Returns `None` when the enable bit
    /// is clear (VITA 49.2 §5.1.6, Rule 5.1.6-6).
    pub fn valid_data_indicator(&self) -> Option<bool> {
        self.valid_data_enabled().then_some(self.0 & (1 << 18) > 0)
    }
    /// Sets the valid data indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 30) and writes `v` to bit 18.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_valid_data_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 30) & !(1 << 18);
        if let Some(v) = value {
            self.0 |= 1 << 30;
            self.0 |= (v as u32) << 18;
        }
    }
    /// Returns the reference lock indicator status if the enable bit (bit 29)
    /// is set.
    ///
    /// When `Some(true)`, all phase-locked loops affecting the data are locked
    /// and stable. When `Some(false)`, at least one PLL is not locked and
    /// stable. Returns `None` when the enable bit is clear (VITA 49.2 §5.1.6,
    /// Rule 5.1.6-7).
    pub fn reference_lock_indicator(&self) -> Option<bool> {
        self.reference_lock_enabled()
            .then_some(self.0 & (1 << 17) > 0)
    }
    /// Sets the reference lock indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 29) and writes `v` to bit 17.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_reference_lock_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 29) & !(1 << 17);
        if let Some(v) = value {
            self.0 |= 1 << 29;
            self.0 |= (v as u32) << 17;
        }
    }
    /// Returns the automatic gain control (AGC/MGC) indicator status if the
    /// enable bit (bit 28) is set.
    ///
    /// When `Some(true)`, AGC is active. When `Some(false)`, MGC (manual gain
    /// control) is in use. Returns `None` when the enable bit is clear
    /// (VITA 49.2 §5.1.6, Rule 5.1.6-8).
    pub fn agc_indicator(&self) -> Option<bool> {
        self.agc_enabled().then_some(self.0 & (1 << 16) > 0)
    }
    /// Sets the AGC/MGC indicator.
    ///
    /// `Some(true)` indicates AGC is active; `Some(false)` indicates MGC.
    /// Sets the enable bit (bit 28) and writes `v` to bit 16.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_agc_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 28) & !(1 << 16);
        if let Some(v) = value {
            self.0 |= 1 << 28;
            self.0 |= (v as u32) << 16;
        }
    }
    /// Returns the detected signal indicator status if the enable bit (bit 27)
    /// is set.
    ///
    /// When `Some(true)`, the payload contains a detected signal. Returns
    /// `None` when the enable bit is clear (VITA 49.2 §5.1.6, Rule 5.1.6-9).
    pub fn detected_signal_indicator(&self) -> Option<bool> {
        self.detected_signal_enabled()
            .then_some(self.0 & (1 << 15) > 0)
    }
    /// Sets the detected signal indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 27) and writes `v` to bit 15.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_detected_signal_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 27) & !(1 << 15);
        if let Some(v) = value {
            self.0 |= 1 << 27;
            self.0 |= (v as u32) << 15;
        }
    }
    /// Returns the spectral inversion indicator status if the enable bit
    /// (bit 26) is set.
    ///
    /// When `Some(true)`, the signal in the payload has an inverted spectrum
    /// relative to the spectrum at the system reference point. Returns `None`
    /// when the enable bit is clear (VITA 49.2 §5.1.6, Rule 5.1.6-10).
    pub fn spectral_inversion_indicator(&self) -> Option<bool> {
        self.spectral_inversion_enabled()
            .then_some(self.0 & (1 << 14) > 0)
    }
    /// Sets the spectral inversion indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 26) and writes `v` to bit 14.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_spectral_inversion_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 26) & !(1 << 14);
        if let Some(v) = value {
            self.0 |= 1 << 26;
            self.0 |= (v as u32) << 14;
        }
    }
    /// Returns the over-range indicator status if the enable bit (bit 25) is
    /// set.
    ///
    /// When `Some(true)`, at least one data sample in the payload is invalid
    /// because the signal exceeded the representable range of the data item.
    /// Returns `None` when the enable bit is clear (VITA 49.2 §5.1.6,
    /// Rule 5.1.6-11).
    pub fn over_range_indicator(&self) -> Option<bool> {
        self.over_range_enabled().then_some(self.0 & (1 << 13) > 0)
    }
    /// Sets the over-range indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 25) and writes `v` to bit 13.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_over_range_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 25) & !(1 << 13);
        if let Some(v) = value {
            self.0 |= 1 << 25;
            self.0 |= (v as u32) << 13;
        }
    }
    /// Returns the sample loss indicator status if the enable bit (bit 24) is
    /// set.
    ///
    /// When `Some(true)`, the packet contains at least one sample discontinuity
    /// caused by processing errors or buffer overflow. Returns `None` when the
    /// enable bit is clear (VITA 49.2 §5.1.6, Rule 5.1.6-12).
    pub fn sample_loss_indicator(&self) -> Option<bool> {
        self.sample_loss_enabled().then_some(self.0 & (1 << 12) > 0)
    }
    /// Sets the sample loss indicator.
    ///
    /// `Some(v)` sets the enable bit (bit 24) and writes `v` to bit 12.
    /// `None` clears both bits, marking the indicator as not present.
    pub fn set_sample_loss_indicator(&mut self, value: Option<bool>) {
        self.0 &= !(1 << 24) & !(1 << 12);
        if let Some(v) = value {
            self.0 |= 1 << 24;
            self.0 |= (v as u32) << 12;
        }
    }
    /// Returns the sample frame indicator if both enable bits (bits 23 and 22)
    /// are set.
    ///
    /// Indicates where this packet sits within the current Sample Frame. Both
    /// enable bits must be `1` for the indicator to be valid. Returns `None`
    /// when either enable bit is clear (VITA 49.2 §5.1.6, Table 5.1.6-1).
    pub fn sample_frame_indicator(&self) -> Option<SampleFrameIndicator> {
        self.sample_frame_enabled()
            .then(|| ((self.0 >> 10) & 0b11).try_into().unwrap_or_else(|_|
                  unreachable!("0b11 mask guarantees a value in 0..=3, all of which are valid SampleFrameIndicator variants")
            ))
    }
    /// Sets the sample frame indicator.
    ///
    /// `Some(v)` sets both enable bits (bits 23 and 22) and writes the 2-bit
    /// indicator value to bits 11–10. `None` clears all four bits, marking the
    /// indicator as not present.
    pub fn set_sample_frame_indicator(&mut self, value: Option<SampleFrameIndicator>) {
        self.0 &= !(0b11 << 22) & !(0b11 << 10);
        if let Some(v) = value {
            self.0 |= 0b11 << 22;
            self.0 |= (v as u32) << 10;
        }
    }
    /// Returns the user-defined indicator bits (bits 9–8) if both user-defined
    /// enable bits (bits 21 and 20) are set.
    ///
    /// The meaning of these bits is application-defined. Returns `None` when
    /// either enable bit is clear (VITA 49.2 §5.1.6, Permission 5.1.6-1,
    /// Rule 5.1.6-4).
    pub fn user_defined_indicator(&self) -> Option<u8> {
        self.user_defined_enabled()
            .then_some(((self.0 >> 8) & 0b11) as u8)
    }
    /// Sets the user-defined indicator.
    ///
    /// `Some(v)` sets both enable bits (bits 21 and 20) and writes the
    /// low 2 bits of `v` to bits 9–8. `None` clears all four bits, marking
    /// the indicator as not present.
    pub fn set_user_defined_indicator(&mut self, value: Option<u8>) {
        debug_assert!(
            value.map_or(true, |n| n <= 3),
            "user-defined indicator must be in 0..=3"
        );
        self.0 &= !(0b11 << 20) & !(0b11 << 8);
        if let Some(v) = value {
            self.0 |= 0b11 << 20;
            self.0 |= (u32::from(v) & 0b11) << 8;
        }
    }
    /// Returns the associated context packet count if the enable bit (`E`,
    /// bit 7) is set.
    ///
    /// When `Some(n)`, `n` is the number of Context packets (0–127) that are
    /// directly or indirectly associated with this Data packet. Returns `None`
    /// when the `E` bit is clear and the count field is undefined
    /// (VITA 49.2 §5.1.6, Rule 5.1.6-13).
    pub fn associated_context_packet_count(&self) -> Option<u8> {
        self.associated_context_packet_count_enabled()
            .then_some((self.0 & 0x7F) as u8)
    }
    /// Sets the associated context packet count.
    ///
    /// `Some(n)` sets the enable bit (`E`, bit 7) and writes the low 7 bits of
    /// `n` to bits 6–0 (valid range 0–127). `None` clears all 8 bits, marking
    /// the count as not present.
    pub fn set_associated_context_packet_count(&mut self, value: Option<u8>) {
        debug_assert!(
            value.map_or(true, |n| n <= 127),
            "associated context packet count must be in 0..=127"
        );
        self.0 &= !0xFF;
        if let Some(n) = value {
            self.0 |= 1 << 7;
            self.0 |= u32::from(n) & 0x7F;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trailer(raw: u32) -> Trailer {
        Trailer(raw)
    }

    /// Bool-indicator helpers
    ///
    /// All eight single-bit indicator fields share the same behavior:
    ///   * enable bit clear: getter returns None
    ///   * enable bit set, ind bit clear: getter returns Some(false)
    ///   * enable bit set, ind bit set: getter returns Some(true)
    ///   * setter(None) zeroes both bits without disturbing any other bit
    macro_rules! test_bool_indicator {
        ($mod:ident, $get:ident, $set:ident, enable = $en:literal, indicator = $ind:literal) => {
            mod $mod {
                use super::*;

                #[test]
                fn disabled_returns_none() {
                    assert_eq!(trailer(0).$get(), None);
                }

                #[test]
                fn enabled_clear_returns_some_false() {
                    assert_eq!(trailer(1u32 << $en).$get(), Some(false));
                }

                #[test]
                fn enabled_set_returns_some_true() {
                    assert_eq!(trailer((1u32 << $en) | (1u32 << $ind)).$get(), Some(true));
                }

                #[test]
                fn setter_round_trips() {
                    let mut t = trailer(0);
                    t.$set(Some(true));
                    assert_eq!(t.$get(), Some(true));
                    t.$set(Some(false));
                    assert_eq!(t.$get(), Some(false));
                    t.$set(None);
                    assert_eq!(t.$get(), None);
                }

                #[test]
                fn setter_none_clears_only_its_bits() {
                    let mask = (1u32 << $en) | (1u32 << $ind);
                    let mut t = trailer(u32::MAX);
                    t.$set(None);
                    assert_eq!(t.as_u32() & mask, 0, "enable/indicator bits not cleared");
                    assert_eq!(
                        t.as_u32() & !mask,
                        u32::MAX & !mask,
                        "unrelated bits were disturbed"
                    );
                }
            }
        };
    }

    test_bool_indicator!(
        cal_time,
        cal_time_indicator,
        set_cal_time_indicator,
        enable = 31,
        indicator = 19
    );
    test_bool_indicator!(
        valid_data,
        valid_data_indicator,
        set_valid_data_indicator,
        enable = 30,
        indicator = 18
    );
    test_bool_indicator!(
        reference_lock,
        reference_lock_indicator,
        set_reference_lock_indicator,
        enable = 29,
        indicator = 17
    );
    test_bool_indicator!(
        agc,
        agc_indicator,
        set_agc_indicator,
        enable = 28,
        indicator = 16
    );
    test_bool_indicator!(
        detected_signal,
        detected_signal_indicator,
        set_detected_signal_indicator,
        enable = 27,
        indicator = 15
    );
    test_bool_indicator!(
        spectral_inversion,
        spectral_inversion_indicator,
        set_spectral_inversion_indicator,
        enable = 26,
        indicator = 14
    );
    test_bool_indicator!(
        over_range,
        over_range_indicator,
        set_over_range_indicator,
        enable = 25,
        indicator = 13
    );
    test_bool_indicator!(
        sample_loss,
        sample_loss_indicator,
        set_sample_loss_indicator,
        enable = 24,
        indicator = 12
    );

    /// Test Sample frame indicator
    ///
    /// Unlike the bool fields, this one requires *two* enable bits (23 and 22)
    /// and stores a 2-bit value in bits 11–10.
    mod sample_frame {
        use super::*;
        use SampleFrameIndicator::*;

        const ENABLE: u32 = (1 << 23) | (1 << 22);
        const MASK: u32 = ENABLE | (0b11 << 10);

        #[test]
        fn disabled_returns_none() {
            assert_eq!(trailer(0).sample_frame_indicator(), None);
        }

        #[test]
        fn only_one_enable_bit_set_returns_none() {
            // Both bits must be set; partial enable is treated as disabled.
            assert_eq!(trailer(1 << 23).sample_frame_indicator(), None);
            assert_eq!(trailer(1 << 22).sample_frame_indicator(), None);
        }

        #[test]
        fn all_variants_round_trip_via_raw_bits() {
            for v in [
                NotApplicable,
                FirstDataPacket,
                MiddleDataPacket,
                FinalDataPacket,
            ] {
                let t = trailer(ENABLE | ((v as u32) << 10));
                assert_eq!(t.sample_frame_indicator(), Some(v), "{v:?}");
            }
        }

        #[test]
        fn setter_round_trips_all_variants() {
            let mut t = trailer(0);
            for v in [
                NotApplicable,
                FirstDataPacket,
                MiddleDataPacket,
                FinalDataPacket,
            ] {
                t.set_sample_frame_indicator(Some(v));
                assert_eq!(t.sample_frame_indicator(), Some(v), "{v:?}");
            }
            t.set_sample_frame_indicator(None);
            assert_eq!(t.sample_frame_indicator(), None);
        }

        #[test]
        fn setter_none_clears_only_its_bits() {
            let mut t = trailer(u32::MAX);
            t.set_sample_frame_indicator(None);
            assert_eq!(t.as_u32() & MASK, 0, "enable/indicator bits not cleared");
            assert_eq!(
                t.as_u32() & !MASK,
                u32::MAX & !MASK,
                "unrelated bits were disturbed"
            );
        }
    }

    /// User-defined indicators tests
    ///
    /// Two enable bits (21 and 20); 2-bit value in bits 9–8.
    /// Only the low 2 bits of the supplied value are stored.
    mod user_defined {
        use super::*;

        const ENABLE: u32 = (1 << 21) | (1 << 20);
        const MASK: u32 = ENABLE | (0b11 << 8);

        #[test]
        fn disabled_returns_none() {
            assert_eq!(trailer(0).user_defined_indicator(), None);
        }

        #[test]
        fn only_one_enable_bit_set_returns_none() {
            assert_eq!(trailer(1 << 21).user_defined_indicator(), None);
            assert_eq!(trailer(1 << 20).user_defined_indicator(), None);
        }

        #[test]
        fn all_two_bit_values_round_trip_via_raw_bits() {
            for v in 0u8..=3 {
                let t = trailer(ENABLE | (u32::from(v) << 8));
                assert_eq!(t.user_defined_indicator(), Some(v), "value={v}");
            }
        }

        #[test]
        fn setter_round_trips_all_two_bit_values() {
            let mut t = trailer(0);
            for v in 0u8..=3 {
                t.set_user_defined_indicator(Some(v));
                assert_eq!(t.user_defined_indicator(), Some(v));
            }
            t.set_user_defined_indicator(None);
            assert_eq!(t.user_defined_indicator(), None);
        }

        #[test]
        fn setter_none_clears_only_its_bits() {
            let mut t = trailer(u32::MAX);
            t.set_user_defined_indicator(None);
            assert_eq!(t.as_u32() & MASK, 0, "enable/indicator bits not cleared");
            assert_eq!(
                t.as_u32() & !MASK,
                u32::MAX & !MASK,
                "unrelated bits were disturbed"
            );
        }
    }

    /// Associated context packet count tests
    ///
    /// Enable bit 7 (the `E` flag); 7-bit unsigned count in bits 6–0 (0–127).
    mod context_packet_count {
        use super::*;

        const MASK: u32 = 0xFF; // bit 7 (enable) + bits 6-0 (count)

        #[test]
        fn disabled_returns_none() {
            assert_eq!(trailer(0).associated_context_packet_count(), None);
        }

        #[test]
        fn count_without_enable_bit_returns_none() {
            // Bits 6-0 set but enable bit 7 clear → still None.
            assert_eq!(trailer(0x7F).associated_context_packet_count(), None);
        }

        #[test]
        fn boundary_values_round_trip_via_raw_bits() {
            for n in [0u8, 1, 63, 127] {
                let t = trailer((1 << 7) | u32::from(n));
                assert_eq!(t.associated_context_packet_count(), Some(n), "n={n}");
            }
        }

        #[test]
        fn setter_round_trips_boundary_values() {
            let mut t = trailer(0);
            for n in [0u8, 1, 63, 127] {
                t.set_associated_context_packet_count(Some(n));
                assert_eq!(t.associated_context_packet_count(), Some(n), "n={n}");
            }
            t.set_associated_context_packet_count(None);
            assert_eq!(t.associated_context_packet_count(), None);
        }

        #[test]
        fn setter_none_clears_only_its_bits() {
            let mut t = trailer(u32::MAX);
            t.set_associated_context_packet_count(None);
            assert_eq!(t.as_u32() & MASK, 0, "enable/count bits not cleared");
            assert_eq!(
                t.as_u32() & !MASK,
                u32::MAX & !MASK,
                "unrelated bits were disturbed"
            );
        }
    }
}
