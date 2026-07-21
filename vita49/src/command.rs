// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to command payloads
(ANSI/VITA-49.2-2017 section 8).
*/

use core::fmt;

use crate::{
    prelude::*, Ack, Cancellation, CommandPayload, Control, ControlAckMode, IdFormat, QueryAck,
};
use deku::prelude::*;

/// Main command payload structure.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, packet_header: &PacketHeader"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command {
    /// Control acknowledgement mode.
    cam: ControlAckMode,
    /// Message ID.
    message_id: u32,
    /// Controllee ID.
    #[deku(cond = "cam.controllee_enabled() && cam.controllee_id_format() == IdFormat::Id32bit")]
    controllee_id: Option<u32>,
    /// Controllee UUID.
    #[deku(
        cond = "cam.controllee_enabled() && cam.controllee_id_format() == IdFormat::Uuid128bit"
    )]
    controllee_uuid: Option<u128>,
    /// Controller ID.
    #[deku(cond = "cam.controller_enabled() && cam.controller_id_format() == IdFormat::Id32bit")]
    controller_id: Option<u32>,
    /// Controller UUID.
    #[deku(
        cond = "cam.controller_enabled() && cam.controller_id_format() == IdFormat::Uuid128bit"
    )]
    controller_uuid: Option<u128>,
    #[deku(ctx = "&cam, packet_header")]
    command_payload: CommandPayload,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            cam: Default::default(),
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::Control(Control::default()),
        }
    }
}

impl Command {
    /// Create a new, empty control packet.
    pub fn new_control() -> Command {
        Command::default()
    }

    /// Create a new, empty cancellation packet.
    pub fn new_cancellation() -> Command {
        Self {
            cam: Default::default(),
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::Cancellation(Cancellation::default()),
        }
    }

    /// Create a new, empty validation ACK packet.
    pub fn new_validation_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_validation();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::ValidationAck(Ack::default()),
        }
    }

    /// Create a new, empty execution ACK packet.
    pub fn new_exec_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_execution();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::ExecAck(Ack::default()),
        }
    }

    /// Create a new, empty query ACK packet.
    pub fn new_query_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_state();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::QueryAck(QueryAck::default()),
        }
    }

    /// Get the packet message ID.
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Set the packet message ID.
    pub fn set_message_id(&mut self, message_id: u32) {
        self.message_id = message_id;
    }

    /// Get the packet's Control Ack Mode (CAM)
    pub fn cam(&self) -> ControlAckMode {
        self.cam
    }

    /// Set the packet's Control Ack Mode (CAM)
    /// # Example
    /// ```
    /// use vita49::{prelude::*, ControlAckMode, ActionMode};
    /// let mut packet = Vrt::new_control_packet();
    /// let command_mut = packet.payload_mut().command_mut().unwrap();
    /// let mut cam = ControlAckMode::default();
    /// cam.set_action_mode(ActionMode::Execute);
    /// command_mut.set_cam(cam);
    /// assert_eq!(command_mut.cam().action_mode(), ActionMode::Execute);
    /// ````
    pub fn set_cam(&mut self, mode: ControlAckMode) {
        self.cam = mode;
    }

    /// Get the controllee identifier.
    pub fn controllee_id(&self) -> Option<u32> {
        self.controllee_id
    }
    /// Sets the controllee identifier. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controllee_uuid` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controllee_id(&mut self, id: Option<u32>) -> Result<(), VitaError> {
        if id.is_some() && self.controllee_uuid.is_some() {
            return Err(VitaError::TriedIdWhenUuidSet);
        }
        self.controllee_id = id;
        if id.is_some() {
            self.cam.enable_controllee();
            self.cam.set_controllee_id_format(IdFormat::Id32bit);
        } else if self.controllee_uuid.is_none() {
            self.cam.disable_controllee();
        }
        Ok(())
    }

    /// Get the controller identifier.
    pub fn controller_id(&self) -> Option<u32> {
        self.controller_id
    }
    /// Sets the controller identifier. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controller_uuid` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controller_id(&mut self, id: Option<u32>) -> Result<(), VitaError> {
        if id.is_some() && self.controller_uuid.is_some() {
            return Err(VitaError::TriedIdWhenUuidSet);
        }
        self.controller_id = id;
        if id.is_some() {
            self.cam.enable_controller();
            self.cam.set_controller_id_format(IdFormat::Id32bit);
        } else if self.controller_uuid.is_none() {
            self.cam.disable_controller();
        }
        Ok(())
    }

    /// Get the controllee UUID.
    pub fn controllee_uuid(&self) -> Option<u128> {
        self.controllee_uuid
    }
    /// Sets the controllee UUID. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controllee_id` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controllee_uuid(&mut self, uuid: Option<u128>) -> Result<(), VitaError> {
        if uuid.is_some() && self.controllee_id.is_some() {
            return Err(VitaError::TriedUuidWhenIdSet);
        }
        self.controllee_uuid = uuid;
        if uuid.is_some() {
            self.cam.enable_controllee();
            self.cam.set_controllee_id_format(IdFormat::Uuid128bit);
        } else if self.controllee_id.is_none() {
            self.cam.disable_controllee();
        }
        Ok(())
    }

    /// Get the controller UUID.
    pub fn controller_uuid(&self) -> Option<u128> {
        self.controller_uuid
    }
    /// Sets the controller UUID. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controller_id` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controller_uuid(&mut self, uuid: Option<u128>) -> Result<(), VitaError> {
        if uuid.is_some() && self.controller_id.is_some() {
            return Err(VitaError::TriedUuidWhenIdSet);
        }
        self.controller_uuid = uuid;
        if uuid.is_some() {
            self.cam.enable_controller();
            self.cam.set_controller_id_format(IdFormat::Uuid128bit);
        } else if self.controller_id.is_none() {
            self.cam.disable_controller();
        }
        Ok(())
    }

    /// Reconcile the CAM warning/error indicator bits with the actual contents
    /// of an acknowledge payload.
    ///
    /// The Warning Indicator Field is present on the wire only when the CAM's
    /// AckW bit is set, and the Error Indicator Field only when its AckEr bit is
    /// set (ANSI/VITA-49.2-2017 8.4.1.1). The per-field setters populate the
    /// WIF/EIF contents but cannot reach the CAM (which lives on the command,
    /// not the ACK payload), so this must run before serialization to keep the
    /// indicator bits consistent with the payload.
    ///
    /// AckEr is applicable only to an Execution Acknowledge; a Validation
    /// Acknowledge must always leave it zero (bit 16). Any error content on a
    /// validation ACK is therefore dropped here so the payload can never
    /// contradict the CAM. [`Vrt::update_packet_size`](Vrt::update_packet_size)
    /// calls this automatically; on a non-ACK payload it is a no-op.
    pub fn sync_ack_cam(&mut self) {
        let (warning, error) = match &mut self.command_payload {
            CommandPayload::ValidationAck(ack) => {
                ack.clear_error_fields();
                (ack.has_warning_fields(), false)
            }
            CommandPayload::ExecAck(ack) => (ack.has_warning_fields(), ack.has_error_fields()),
            _ => return,
        };
        if warning {
            self.cam.set_warning();
        } else {
            self.cam.unset_warning();
        }
        if error {
            self.cam.set_error();
        } else {
            self.cam.unset_error();
        }
    }

    /// Get a reference to the underlying command payload enumeration.
    pub fn payload(&self) -> &CommandPayload {
        &self.command_payload
    }

    /// Get a mutable reference to the underlying command payload enumeration.
    pub fn payload_mut(&mut self) -> &mut CommandPayload {
        &mut self.command_payload
    }

    /// Get the size of the command packet (in 32-bit words).
    pub fn size_words(&self) -> u16 {
        let mut ret = self.cam.size_words();
        ret += 1; // message_id
        if self.controllee_id.is_some() {
            ret += 1;
        } else if self.controllee_uuid.is_some() {
            ret += 4;
        }
        if self.controller_id.is_some() {
            ret += 1;
        } else if self.controller_uuid.is_some() {
            ret += 4;
        }
        ret += self.command_payload.size_words();
        ret
    }
}

impl TryFrom<Payload> for Command {
    type Error = Payload;

    fn try_from(value: Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Command(c) => Ok(c),
            a => Err(a),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cam)?;
        writeln!(f, "Message ID: {:x}", self.message_id)?;
        if let Some(cid) = self.controllee_id {
            writeln!(f, "Controllee ID: {cid:x}")?;
        }
        if let Some(cuuid) = self.controllee_uuid {
            writeln!(f, "Controllee UUID: {cuuid:x}")?;
        }
        if let Some(cid) = self.controller_id {
            writeln!(f, "Controller ID: {cid:x}")?;
        }
        if let Some(cuuid) = self.controller_uuid {
            writeln!(f, "Controller UUID: {cuuid:x}")?;
        }
        match &self.command_payload {
            CommandPayload::Control(p) => write!(f, "{p}")?,
            CommandPayload::Cancellation(p) => write!(f, "{p}")?,
            CommandPayload::ValidationAck(p) => write!(f, "Validation {p}")?,
            CommandPayload::ExecAck(p) => write!(f, "Execution {p}")?,
            CommandPayload::QueryAck(p) => write!(f, "{p}")?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::{ActionMode, ControlAckMode, IdFormat, Tsf, Tsi};

    #[test]
    fn create_control_packet() {
        let mut packet = Vrt::new_control_packet();
        packet.set_stream_id(Some(0xDEADBEEF));
        packet.set_integer_timestamp(Some(0), Tsi::Utc).unwrap();
        packet
            .set_fractional_timestamp(Some(0), Tsf::SampleCount)
            .unwrap();
        let command = packet.payload_mut().command_mut().unwrap();
        command.set_message_id(123);
        let mut cam = ControlAckMode::default();
        cam.enable_controllee();
        cam.enable_controller();
        cam.set_controllee_id_format(IdFormat::Id32bit);
        cam.set_controller_id_format(IdFormat::Uuid128bit);
        cam.set_action_mode(ActionMode::Execute);
        cam.set_partial_packet_impl_permitted();
        cam.set_warnings_permitted();
        cam.set_validation();
        cam.set_warning();
        cam.set_error();
        command.set_cam(cam);
        command.controllee_id = Some(123);
        command.controller_uuid = Some(321);
    }

    #[test]
    fn exec_ack_with_warning_and_error_fields_round_trips() {
        use crate::command_prelude::*;
        // An Execution Acknowledge may carry both warnings and errors. Before the
        // CAM was reconciled with the payload, the AckW/AckEr bits stayed clear,
        // so the WIF/EIF contents desynced the stream on re-parse.
        let mut packet = Vrt::new_exec_ack_packet();
        {
            let ack = packet
                .payload_mut()
                .command_mut()
                .unwrap()
                .payload_mut()
                .exec_ack_mut()
                .unwrap();
            let mut warn = AckResponse::default();
            warn.set_param_out_of_range();
            ack.set_bandwidth(AckLevel::Warning, Some(warn));
            let mut err = AckResponse::default();
            err.set_param_out_of_range();
            ack.set_bandwidth(AckLevel::Error, Some(err));
        }
        packet.update_packet_size();

        // The CAM now advertises both a warning and an error indicator field.
        let cam = packet.payload().command().unwrap().cam();
        assert!(cam.warning(), "AckW bit must be set when a WIF is present");
        assert!(cam.error(), "AckEr bit must be set when an EIF is present");

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        assert_eq!(
            parsed, packet,
            "ACK with WIF and EIF must round-trip exactly"
        );
    }

    #[test]
    fn validation_ack_warning_leaves_acker_clear_and_round_trips() {
        use crate::command_prelude::*;
        // A Validation Acknowledge may report warnings (AckW) but never errors
        // (AckEr is not applicable to AckV, bit 16).
        let mut packet = Vrt::new_validation_ack_packet();
        {
            let ack = packet
                .payload_mut()
                .command_mut()
                .unwrap()
                .payload_mut()
                .validation_ack_mut()
                .unwrap();
            let mut warn = AckResponse::default();
            warn.set_param_out_of_range();
            ack.set_bandwidth(AckLevel::Warning, Some(warn));
        }
        packet.update_packet_size();

        let cam = packet.payload().command().unwrap().cam();
        assert!(cam.warning(), "AckW must be set for a warning-only ACK");
        assert!(!cam.error(), "AckEr must stay clear on a validation ACK");

        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        assert_eq!(parsed, packet);
    }

    #[test]
    fn validation_ack_drops_inapplicable_error_content() {
        use crate::command_prelude::*;
        // Setting an error field on a validation ACK is a misuse: AckEr is not
        // applicable to AckV, so the error content is normalised away before
        // serialization rather than producing a non-conformant (or desynced)
        // packet.
        let mut packet = Vrt::new_validation_ack_packet();
        {
            let ack = packet
                .payload_mut()
                .command_mut()
                .unwrap()
                .payload_mut()
                .validation_ack_mut()
                .unwrap();
            let mut err = AckResponse::default();
            err.set_param_out_of_range();
            ack.set_bandwidth(AckLevel::Error, Some(err));
        }
        packet.update_packet_size();

        let command = packet.payload().command().unwrap();
        assert!(
            !command.cam().error(),
            "AckEr must be clear on a validation ACK"
        );
        assert!(
            command
                .payload()
                .validation_ack()
                .unwrap()
                .bandwidth()
                .is_none(),
            "the inapplicable error field must be dropped"
        );
        let parsed = Vrt::try_from(packet.to_bytes().unwrap().as_ref()).unwrap();
        assert_eq!(parsed, packet);
    }

    #[test]
    fn clearing_the_last_ack_field_clears_its_cam_bit() {
        use crate::command_prelude::*;
        let mut packet = Vrt::new_exec_ack_packet();
        {
            let ack = packet
                .payload_mut()
                .command_mut()
                .unwrap()
                .payload_mut()
                .exec_ack_mut()
                .unwrap();
            let mut err = AckResponse::default();
            err.set_param_out_of_range();
            ack.set_bandwidth(AckLevel::Error, Some(err));
        }
        packet.update_packet_size();
        assert!(packet.payload().command().unwrap().cam().error());

        // Removing the only error field must clear AckEr again.
        packet
            .payload_mut()
            .command_mut()
            .unwrap()
            .payload_mut()
            .exec_ack_mut()
            .unwrap()
            .set_bandwidth(AckLevel::Error, None);
        packet.update_packet_size();
        assert!(
            !packet.payload().command().unwrap().cam().error(),
            "AckEr must clear once the last error field is removed"
        );
    }

    #[test]
    fn new_ack_for_echoes_identity_and_round_trips() {
        use crate::command_prelude::*;
        let mut request = Vrt::new_control_packet();
        request.set_stream_id(Some(0x1234));
        let command = request.payload_mut().command_mut().unwrap();
        command.set_message_id(0xAABB);
        command.set_controller_id(Some(9)).unwrap();

        let ack = Vrt::new_ack_for(&request, AckKind::Execution).unwrap();
        let parsed = Vrt::try_from(ack.to_bytes().unwrap().as_ref()).unwrap();
        assert_eq!(parsed, ack);
        let ack_command = parsed.payload().command().unwrap();
        assert_eq!(ack_command.message_id(), 0xAABB);
        assert_eq!(ack_command.controller_id(), Some(9));
        assert_eq!(parsed.stream_id(), Some(0x1234));
    }

    #[test]
    fn new_ack_for_rejects_non_command_and_ack_requests() {
        use crate::command_prelude::*;
        // A signal-data packet is not a command and cannot be acknowledged.
        let signal = Vrt::new_signal_data_packet();
        assert!(Vrt::new_ack_for(&signal, AckKind::Validation).is_err());
        // An acknowledge packet cannot itself be acknowledged.
        let ack = Vrt::new_validation_ack_packet();
        assert!(Vrt::new_ack_for(&ack, AckKind::Validation).is_err());
    }
}
