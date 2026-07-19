// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the formatted GPS format
(ANSI/VITA-49.2-2017 section 9.4.5).
*/

use crate::packet_header::{Tsf, Tsi};
use deku::prelude::*;
use fixed::types::extra::{U16, U22, U5};
use fixed::FixedI32;

/// Sentinel for an unknown angle/distance sub-field (Rule 9.4.5-18).
const UNKNOWN: i32 = 0x7FFF_FFFF;
/// Sentinel for an unused Timestamp-of-Position-Fix word (Rule 9.4.5-6).
const TS_UNUSED: u32 = 0xFFFF_FFFF;
/// Manufacturer OUI value meaning "unknown" (Permission 9.4.5-2).
const OUI_UNKNOWN: u32 = 0x00FF_FFFF;

/// Generate a paired getter/setter for a radix-encoded `i32` sub-field. `None` maps to/from
/// the [`UNKNOWN`] sentinel; otherwise the value is a two's-complement fixed-point with the
/// binary point right of bit `Frac`.
macro_rules! radix_field {
    ($get:ident, $set:ident, $field:ident, $frac:ty, $unit:literal) => {
        #[doc = concat!("Get the ", $unit, " (`None` when unknown).")]
        pub fn $get(&self) -> Option<f64> {
            (self.$field != UNKNOWN).then(|| FixedI32::<$frac>::from_bits(self.$field).to_num())
        }
        #[doc = concat!("Set the ", $unit, " (`None` sets the unknown sentinel).")]
        #[doc = ""]
        #[doc = "# Panics"]
        #[doc = ""]
        #[doc = "Panics if the value is outside the field's representable range."]
        pub fn $set(&mut self, v: Option<f64>) {
            self.$field = match v {
                Some(x) => FixedI32::<$frac>::from_num(x).to_bits(),
                None => UNKNOWN,
            };
        }
    };
}

/// Base formatted GPS data structure.
///
/// # Example
///
/// ```
/// use vita49::FormattedGps;
/// use vita49::prelude::Tsi;
///
/// // A default value starts with every field "unknown"; fill in what's known.
/// let mut gps = FormattedGps::default();
/// gps.set_manufacturer_oui(0x0A_1B2C);
/// gps.set_tsi(Tsi::Gps);
/// gps.set_latitude_deg(Some(-31.953_512));
/// gps.set_longitude_deg(Some(115.857_048));
///
/// assert_eq!(gps.tsi(), Tsi::Gps);
/// assert!(gps.latitude_deg().is_some());
/// assert_eq!(gps.altitude_m(), None); // never set, so still unknown
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormattedGps {
    w1: u32,
    ts1: u32,
    ts2: u32,
    ts3: u32,
    latitude: i32,
    longitude: i32,
    altitude: i32,
    speed_over_ground: i32,
    heading_angle: i32,
    track_angle: i32,
    magnetic_variation: i32,
}

impl Default for FormattedGps {
    /// A field with every value unknown: OUI = `FF-FF-FF`, TSI/TSF = Null, the position-fix
    /// timestamp unused, and every geolocation sub-field the unknown sentinel. Fill in what's
    /// known with the setters — a partial fix stays valid.
    fn default() -> Self {
        Self {
            w1: OUI_UNKNOWN, // TSI = TSF = Null (0), OUI = unknown
            ts1: TS_UNUSED,
            ts2: TS_UNUSED,
            ts3: TS_UNUSED,
            latitude: UNKNOWN,
            longitude: UNKNOWN,
            altitude: UNKNOWN,
            speed_over_ground: UNKNOWN,
            heading_angle: UNKNOWN,
            track_angle: UNKNOWN,
            magnetic_variation: UNKNOWN,
        }
    }
}

impl FormattedGps {
    /// Gets the size of the formatted GPS structure in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (std::mem::size_of_val(self) / std::mem::size_of::<u32>()) as u16
    }

    /// The GPS/INS Manufacturer OUI (Rule 9.4.5-2); `FF-FF-FF` when unknown (Perm 9.4.5-2).
    pub fn manufacturer_oui(&self) -> u32 {
        self.w1 & 0x00FF_FFFF
    }
    /// Set the GPS/INS Manufacturer OUI (low 24 bits used).
    pub fn set_manufacturer_oui(&mut self, oui: u32) {
        self.w1 = (self.w1 & 0xFF00_0000) | (oui & 0x00FF_FFFF);
    }

    /// The integer-seconds Timestamp-of-Position-Fix type (Rule 9.4.5-3, Table 9.4.5-1).
    pub fn tsi(&self) -> Tsi {
        // The 2-bit field spans all four Tsi variants, so the conversion is infallible.
        (((self.w1 >> 26) & 0b11) as u8).try_into().unwrap()
    }
    /// Set the integer-seconds Timestamp-of-Position-Fix type.
    pub fn set_tsi(&mut self, tsi: Tsi) {
        self.w1 = (self.w1 & !(0b11 << 26)) | ((tsi as u32 & 0b11) << 26);
    }

    /// The fractional-seconds Timestamp-of-Position-Fix type (Rule 9.4.5-4, Table 9.4.5-2).
    pub fn tsf(&self) -> Tsf {
        // The 2-bit field spans all four Tsf variants, so the conversion is infallible.
        (((self.w1 >> 24) & 0b11) as u8).try_into().unwrap()
    }
    /// Set the fractional-seconds Timestamp-of-Position-Fix type.
    pub fn set_tsf(&mut self, tsf: Tsf) {
        self.w1 = (self.w1 & !(0b11 << 24)) | ((tsf as u32 & 0b11) << 24);
    }

    /// The integer-second Timestamp of the position fix (meaningful when [`tsi`](Self::tsi)
    /// is non-Null).
    pub fn integer_timestamp(&self) -> u32 {
        self.ts1
    }
    /// Set the integer-second Timestamp of the position fix.
    pub fn set_integer_timestamp(&mut self, seconds: u32) {
        self.ts1 = seconds;
    }

    /// The 64-bit fractional-second Timestamp of the position fix.
    pub fn fractional_timestamp(&self) -> u64 {
        (u64::from(self.ts2) << 32) | u64::from(self.ts3)
    }
    /// Set the 64-bit fractional-second Timestamp of the position fix.
    pub fn set_fractional_timestamp(&mut self, frac: u64) {
        self.ts2 = (frac >> 32) as u32;
        self.ts3 = frac as u32;
    }

    // Geolocation Angle Format = 32-bit two's-complement, radix point right of bit 22
    // (Def 9.4.5-1); altitude radix 5 (Rule 9.4.5-10); speed radix 16 (Rule 9.4.5-11).
    radix_field!(
        latitude_deg,
        set_latitude_deg,
        latitude,
        U22,
        "latitude in degrees"
    );
    radix_field!(
        longitude_deg,
        set_longitude_deg,
        longitude,
        U22,
        "longitude in degrees"
    );
    radix_field!(
        altitude_m,
        set_altitude_m,
        altitude,
        U5,
        "altitude in meters"
    );
    radix_field!(
        speed_over_ground_mps,
        set_speed_over_ground_mps,
        speed_over_ground,
        U16,
        "speed over ground in meters/second"
    );
    radix_field!(
        heading_angle_deg,
        set_heading_angle_deg,
        heading_angle,
        U22,
        "heading angle in degrees"
    );
    radix_field!(
        track_angle_deg,
        set_track_angle_deg,
        track_angle,
        U22,
        "track angle in degrees"
    );
    radix_field!(
        magnetic_variation_deg,
        set_magnetic_variation_deg,
        magnetic_variation,
        U22,
        "magnetic variation in degrees"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_unknown() {
        let g = FormattedGps::default();
        assert_eq!(g.manufacturer_oui(), OUI_UNKNOWN);
        assert_eq!(g.tsi(), Tsi::Null);
        assert_eq!(g.tsf(), Tsf::Null);
        assert_eq!(g.integer_timestamp(), TS_UNUSED);
        for v in [
            g.latitude_deg(),
            g.longitude_deg(),
            g.altitude_m(),
            g.speed_over_ground_mps(),
            g.heading_angle_deg(),
            g.track_angle_deg(),
            g.magnetic_variation_deg(),
        ] {
            assert_eq!(v, None);
        }
    }

    #[test]
    fn header_and_position_round_trip() {
        let mut g = FormattedGps::default();
        g.set_manufacturer_oui(0x0A_1B2C);
        g.set_tsi(Tsi::Other);
        g.set_tsf(Tsf::SampleCount);
        g.set_integer_timestamp(1_700_000_000);
        g.set_fractional_timestamp(0x1122_3344_5566_7788);
        g.set_latitude_deg(Some(-31.953_512));
        g.set_longitude_deg(Some(115.857_048));
        g.set_altitude_m(Some(21.0));
        g.set_speed_over_ground_mps(Some(3.5));

        assert_eq!(g.manufacturer_oui(), 0x0A_1B2C);
        assert_eq!(g.tsi(), Tsi::Other);
        assert_eq!(g.tsf(), Tsf::SampleCount);
        assert_eq!(g.integer_timestamp(), 1_700_000_000);
        assert_eq!(g.fractional_timestamp(), 0x1122_3344_5566_7788);
        // radix-22 angle resolution ≈ 2.4e-7°, radix-5 alt ≈ 0.03 m, radix-16 speed ≈ 1.5e-5
        assert!((g.latitude_deg().unwrap() - -31.953_512).abs() < 1e-6);
        assert!((g.longitude_deg().unwrap() - 115.857_048).abs() < 1e-6);
        assert!((g.altitude_m().unwrap() - 21.0).abs() < 0.05);
        assert!((g.speed_over_ground_mps().unwrap() - 3.5).abs() < 1e-3);
        // untouched fields still unknown
        assert_eq!(g.heading_angle_deg(), None);
        assert_eq!(g.track_angle_deg(), None);
    }

    #[test]
    fn wide_but_in_range_round_trips() {
        let mut g = FormattedGps::default();
        // 300° is out of navigational range but within the ±512° Geolocation Angle Format.
        g.set_latitude_deg(Some(300.0));
        assert!((g.latitude_deg().unwrap() - 300.0).abs() < 1e-4);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn out_of_range_panics() {
        // Past the field's representable range the caller is responsible for the value —
        // `set_*` panics rather than silently clamping.
        FormattedGps::default().set_altitude_m(Some(1.0e12));
    }
}
