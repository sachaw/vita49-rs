// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the ECEF ephemeris format
(ANSI/VITA-49.2-2017 section 9.4.3).
*/

use crate::packet_header::{Tsf, Tsi};
use deku::prelude::*;
use fixed::types::extra::{U16, U22, U5};
use fixed::FixedI32;

/// Sentinel for an unknown position/attitude/velocity sub-field (Rule 9.4.3-13).
const UNKNOWN: i32 = 0x7FFF_FFFF;
/// Sentinel for an unused Timestamp-of-Position-Fix word (Rule 9.4.3-7).
const TS_UNUSED: u32 = 0xFFFF_FFFF;
/// Manufacturer OUI value meaning "unknown" (Permission 9.4.3-2).
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

/// Base ECEF ephemeris data structure.
///
/// The Earth-Centered Earth-Fixed (ECEF) ephemeris (§9.4.3) reports the position, attitude,
/// and velocity of a reference point in the WGS-84 ECEF frame. Position is meters (radix
/// point right of bit 5), velocity meters/second (radix point right of bit 16), and attitude
/// degrees in the Geolocation Angle Format (radix point right of bit 22).
///
/// # Example
///
/// ```
/// use vita49::EcefEphemeris;
/// use vita49::prelude::Tsi;
///
/// // A default value starts with every field "unknown"; fill in what's known.
/// let mut eph = EcefEphemeris::default();
/// eph.set_manufacturer_oui(0x0A_1B2C);
/// eph.set_tsi(Tsi::Other);
/// eph.set_position_x_m(Some(-2362417.0));
/// eph.set_velocity_dx_mps(Some(0.84));
///
/// assert_eq!(eph.tsi(), Tsi::Other);
/// assert!(eph.position_x_m().is_some());
/// assert_eq!(eph.attitude_alpha_deg(), None); // never set, so still unknown
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EcefEphemeris {
    w1: u32,
    ts1: u32,
    ts2: u32,
    ts3: u32,
    position_x: i32,
    position_y: i32,
    position_z: i32,
    attitude_alpha: i32,
    attitude_beta: i32,
    attitude_phi: i32,
    velocity_dx: i32,
    velocity_dy: i32,
    velocity_dz: i32,
}

impl Default for EcefEphemeris {
    /// A field with every value unknown: OUI = `FF-FF-FF`, TSI/TSF = Null, the position-fix
    /// timestamp unused, and every position/attitude/velocity sub-field the unknown sentinel.
    /// Fill in what's known with the setters — a partial ephemeris stays valid.
    fn default() -> Self {
        Self {
            w1: OUI_UNKNOWN, // TSI = TSF = Null (0), OUI = unknown
            ts1: TS_UNUSED,
            ts2: TS_UNUSED,
            ts3: TS_UNUSED,
            position_x: UNKNOWN,
            position_y: UNKNOWN,
            position_z: UNKNOWN,
            attitude_alpha: UNKNOWN,
            attitude_beta: UNKNOWN,
            attitude_phi: UNKNOWN,
            velocity_dx: UNKNOWN,
            velocity_dy: UNKNOWN,
            velocity_dz: UNKNOWN,
        }
    }
}

impl EcefEphemeris {
    /// Gets the size of the ECEF ephemeris field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (std::mem::size_of_val(self) / std::mem::size_of::<u32>()) as u16
    }

    /// The Ephemeris Manufacturer OUI (Rule 9.4.3-2); `FF-FF-FF` when unknown (Perm 9.4.3-2).
    pub fn manufacturer_oui(&self) -> u32 {
        self.w1 & 0x00FF_FFFF
    }
    /// Set the Ephemeris Manufacturer OUI (low 24 bits used).
    pub fn set_manufacturer_oui(&mut self, oui: u32) {
        self.w1 = (self.w1 & 0xFF00_0000) | (oui & 0x00FF_FFFF);
    }

    /// The integer-seconds Timestamp-of-Position-Fix type (Rule 9.4.3-3). Same w1 bit layout
    /// as the Formatted GPS field: TSI in bits 27:26.
    pub fn tsi(&self) -> Tsi {
        // The 2-bit field spans all four Tsi variants, so the conversion is infallible.
        (((self.w1 >> 26) & 0b11) as u8).try_into().unwrap()
    }
    /// Set the integer-seconds Timestamp-of-Position-Fix type.
    pub fn set_tsi(&mut self, tsi: Tsi) {
        self.w1 = (self.w1 & !(0b11 << 26)) | ((tsi as u32 & 0b11) << 26);
    }

    /// The fractional-seconds Timestamp-of-Position-Fix type (Rule 9.4.3-4). TSF in bits 25:24.
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

    // §9.4.3 sub-field radix points: position X/Y/Z in meters, radix point right of bit 5
    // (Rule 9.4.3-9); attitude alpha/beta/phi in the Geolocation Angle Format, radix point
    // right of bit 22 (Rule 9.4.3-11); velocity dX/dY/dZ in meters/second, radix point right
    // of bit 16 (Rule 9.4.3-12). Earth-radius ECEF coordinates (~6.4e6 m) sit well inside the
    // radix-5 integer range (±2^26 ≈ ±6.7e7 m).
    radix_field!(
        position_x_m,
        set_position_x_m,
        position_x,
        U5,
        "ECEF X position in meters"
    );
    radix_field!(
        position_y_m,
        set_position_y_m,
        position_y,
        U5,
        "ECEF Y position in meters"
    );
    radix_field!(
        position_z_m,
        set_position_z_m,
        position_z,
        U5,
        "ECEF Z position in meters"
    );
    radix_field!(
        attitude_alpha_deg,
        set_attitude_alpha_deg,
        attitude_alpha,
        U22,
        "attitude alpha (about Z) in degrees"
    );
    radix_field!(
        attitude_beta_deg,
        set_attitude_beta_deg,
        attitude_beta,
        U22,
        "attitude beta (about Y) in degrees"
    );
    radix_field!(
        attitude_phi_deg,
        set_attitude_phi_deg,
        attitude_phi,
        U22,
        "attitude phi (about X) in degrees"
    );
    radix_field!(
        velocity_dx_mps,
        set_velocity_dx_mps,
        velocity_dx,
        U16,
        "ECEF X velocity in meters/second"
    );
    radix_field!(
        velocity_dy_mps,
        set_velocity_dy_mps,
        velocity_dy,
        U16,
        "ECEF Y velocity in meters/second"
    );
    radix_field!(
        velocity_dz_mps,
        set_velocity_dz_mps,
        velocity_dz,
        U16,
        "ECEF Z velocity in meters/second"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_unknown() {
        let e = EcefEphemeris::default();
        assert_eq!(e.manufacturer_oui(), OUI_UNKNOWN);
        assert_eq!(e.tsi(), Tsi::Null);
        assert_eq!(e.tsf(), Tsf::Null);
        assert_eq!(e.integer_timestamp(), TS_UNUSED);
        for v in [
            e.position_x_m(),
            e.position_y_m(),
            e.position_z_m(),
            e.attitude_alpha_deg(),
            e.attitude_beta_deg(),
            e.attitude_phi_deg(),
            e.velocity_dx_mps(),
            e.velocity_dy_mps(),
            e.velocity_dz_mps(),
        ] {
            assert_eq!(v, None);
        }
    }

    #[test]
    fn header_position_velocity_round_trip() {
        let mut e = EcefEphemeris::default();
        e.set_manufacturer_oui(0x0A_1B2C);
        e.set_tsi(Tsi::Other);
        e.set_tsf(Tsf::SampleCount);
        e.set_integer_timestamp(1_700_000_000);
        e.set_fractional_timestamp(0x1122_3344_5566_7788);
        // A Perth-surface ECEF coordinate (WGS-84) and a small velocity.
        e.set_position_x_m(Some(-2362416.9957));
        e.set_position_y_m(Some(4874502.2828));
        e.set_position_z_m(Some(-3356069.8481));
        e.set_velocity_dx_mps(Some(0.842_413));
        e.set_velocity_dy_mps(Some(1.701_166));
        e.set_velocity_dz_mps(Some(2.810_049));

        assert_eq!(e.manufacturer_oui(), 0x0A_1B2C);
        assert_eq!(e.tsi(), Tsi::Other);
        assert_eq!(e.tsf(), Tsf::SampleCount);
        assert_eq!(e.integer_timestamp(), 1_700_000_000);
        assert_eq!(e.fractional_timestamp(), 0x1122_3344_5566_7788);
        // radix-5 position resolution ≈ 0.03 m, radix-16 velocity ≈ 1.5e-5 m/s
        assert!((e.position_x_m().unwrap() - -2362416.9957).abs() < 0.05);
        assert!((e.position_y_m().unwrap() - 4874502.2828).abs() < 0.05);
        assert!((e.position_z_m().unwrap() - -3356069.8481).abs() < 0.05);
        assert!((e.velocity_dx_mps().unwrap() - 0.842_413).abs() < 1e-3);
        assert!((e.velocity_dy_mps().unwrap() - 1.701_166).abs() < 1e-3);
        assert!((e.velocity_dz_mps().unwrap() - 2.810_049).abs() < 1e-3);
        // untouched attitude fields still unknown (no IMU)
        assert_eq!(e.attitude_alpha_deg(), None);
        assert_eq!(e.attitude_beta_deg(), None);
        assert_eq!(e.attitude_phi_deg(), None);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn out_of_range_panics() {
        // Past the field's representable range (±2^26 m for the radix-5 position) the caller is
        // responsible for the value — `set_*` panics rather than silently clamping.
        EcefEphemeris::default().set_position_x_m(Some(1.0e12));
    }
}
