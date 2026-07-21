// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{cif7::Cif7Opts, prelude::*};
use deku::prelude::*;
use std::fmt;

/// Data structure for control packets. Very similar to [`Context`], but reversed. All the same
/// fields are used, but processed by a controllee to *set* fields rather than report the current
/// value.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, packet_header: &crate::packet_header::PacketHeader"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Control {
    /// CIF0 indicator fields.
    cif0: Cif0,
    /// CIF1 indicator fields.
    #[deku(cond = "cif0.cif1_enabled()")]
    cif1: Option<Cif1>,
    /// CIF2 indicator fields.
    #[deku(cond = "cif0.cif2_enabled()")]
    cif2: Option<Cif2>,
    /// CIF3 indicator fields.
    #[deku(cond = "cif0.cif3_enabled()")]
    cif3: Option<Cif3>,
    /// CIF7 indicator fields.
    #[deku(cond = "cif0.field_attributes_enabled()")]
    pub cif7: Option<Cif7>,

    /// CIF0 data fields.
    #[deku(ctx = "cif0, Cif7Opts::from(cif7.as_ref()), packet_header.prologue()")]
    cif0_fields: Cif0Fields,
    /// CIF1 data fields.
    #[deku(
        cond = "cif0.cif1_enabled()",
        ctx = "cif1.as_ref(), Cif7Opts::from(cif7.as_ref()), packet_header.prologue()"
    )]
    cif1_fields: Option<Cif1Fields>,
    /// CIF2 data fields.
    #[deku(
        cond = "cif0.cif2_enabled()",
        ctx = "cif2.as_ref(), Cif7Opts::from(cif7.as_ref()), packet_header.prologue()"
    )]
    cif2_fields: Option<Cif2Fields>,
    /// CIF3 data fields.
    #[deku(
        cond = "cif0.cif3_enabled()",
        ctx = "cif3.as_ref(), Cif7Opts::from(cif7.as_ref()), packet_header.prologue()"
    )]
    cif3_fields: Option<Cif3Fields>,
}

impl Control {
    /// Get the size of the control structure (in 32-bit words).
    pub fn size_words(&self) -> u16 {
        // Start with 1 32-bit word for the CIF0 field
        let mut ret = 1 + self.cif0_fields.size_words();
        if let Some(f) = &self.cif1_fields {
            ret += 1 + f.size_words();
        }
        if let Some(f) = &self.cif2_fields {
            ret += 1 + f.size_words();
        }
        if let Some(f) = &self.cif3_fields {
            ret += 1 + f.size_words();
        }
        if self.cif0.field_attributes_enabled() {
            ret += 1;
        }
        ret
    }
}

impl Cif0Manipulators for Control {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif0_fields(&self) -> &Cif0Fields {
        &self.cif0_fields
    }
    fn cif0_fields_mut(&mut self) -> &mut Cif0Fields {
        &mut self.cif0_fields
    }
}

impl Cif1Manipulators for Control {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif1(&self) -> Option<&Cif1> {
        self.cif1.as_ref()
    }
    fn cif1_mut(&mut self) -> &mut Option<Cif1> {
        &mut self.cif1
    }
    fn cif1_fields(&self) -> Option<&Cif1Fields> {
        self.cif1_fields.as_ref()
    }
    fn cif1_fields_mut(&mut self) -> &mut Option<Cif1Fields> {
        &mut self.cif1_fields
    }
}

impl Cif2Manipulators for Control {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif2(&self) -> Option<&Cif2> {
        self.cif2.as_ref()
    }
    fn cif2_mut(&mut self) -> &mut Option<Cif2> {
        &mut self.cif2
    }
    fn cif2_fields(&self) -> Option<&Cif2Fields> {
        self.cif2_fields.as_ref()
    }
    fn cif2_fields_mut(&mut self) -> &mut Option<Cif2Fields> {
        &mut self.cif2_fields
    }
}

impl Cif3Manipulators for Control {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif3(&self) -> Option<&Cif3> {
        self.cif3.as_ref()
    }
    fn cif3_mut(&mut self) -> &mut Option<Cif3> {
        &mut self.cif3
    }
    fn cif3_fields(&self) -> Option<&Cif3Fields> {
        self.cif3_fields.as_ref()
    }
    fn cif3_fields_mut(&mut self) -> &mut Option<Cif3Fields> {
        &mut self.cif3_fields
    }
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Control:")?;
        if let Some(bw) = &self.bandwidth_hz() {
            writeln!(f, "  Bandwidth: {bw} Hz")?;
        }
        if let Some(rf_freq) = &self.rf_ref_freq_hz() {
            writeln!(f, "  RF reference frequency: {rf_freq} Hz")?;
        }
        if let Some(samp_rate) = &self.sample_rate_sps() {
            writeln!(f, "  Sample rate: {samp_rate} sps")?;
        }
        if let Some(device_id) = &self.device_id() {
            write!(f, "{device_id}")?;
        }
        if let Some(spectrum) = self.spectrum() {
            write!(f, "{spectrum}")?;
        }
        Ok(())
    }
}
