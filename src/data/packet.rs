use crate::data::net::{PlaybackPacket, NetworkGUID};
use crate::ErrorKind;
use std::io::Read;
use crate::uetypes::{ChannelName, ChannelCloseReason, UEReadExt, UnrealName};
use byteorder::{ReadBytesExt, LE};
use crate::strum::AsStaticRef;
use crate::data::BitReader;

#[derive(Default, Debug, Clone)]
struct DataBunch {
    packet_id: i32,
    ch_index: u32,
    ch_name: ChannelName,
    ch_seq: i32,
    b_open: bool,
    b_close: bool,
    b_is_replication_paused: bool,
    b_is_reliable: bool,
    b_partial: bool,
    b_partial_initial: bool,
    b_partial_final: bool,
    b_has_package_map_exports: bool,
    b_has_must_be_mapped_guids: bool,
    b_ignore_rpcs: bool,
    b_dormant: bool,
    close_reason: ChannelCloseReason,
    data: Vec<u8>,
    data_bit_size: usize, // represent data in bits, so we dont read emptiness
}

#[derive(Default)]
struct UChannel {
    name: ChannelName,
    index: u32,
    actor: Option<Actor>
}

pub struct PacketParser {
    packet_index: i32, // 0
    in_reliable: i32, // 0
    channels: Vec<Option<UChannel>>, //32767
    partial_bunch: Option<DataBunch>
}

// x, y, z
#[derive(Debug, Default, PartialEq)]
pub struct FVector(pub f32, pub f32, pub f32);
// pitch, yaw, roll
#[derive(Debug, Default, PartialEq)]
pub struct FRotator(pub f32, pub f32, pub f32);

#[derive(Debug, Default)]
struct Actor {
    actor_net_guid: NetworkGUID,
    archetype: NetworkGUID,
    level: NetworkGUID,
    location: FVector,
    rotation: FRotator,
    scale: FVector,
    velocity: FVector
}

impl PacketParser {
    pub fn new() -> PacketParser {
        let mut vec: Vec<Option<UChannel>> = Vec::new(); // we have to do this or we have to implement trait which will cause stack overflow
        for x in 0..32767 {
            vec.push(Option::None);
        }
        PacketParser {
            packet_index: 0,
            in_reliable: 0,
            channels: vec,
            partial_bunch: Option::None
        }
    }

    //#[inline]
    pub fn received_raw_packet(&mut self, packet: &PlaybackPacket) -> crate::Result<()> {
        let mut last_byte = packet.data[packet.data.len() - 1];
        if last_byte != 0 {
            let mut bit_size = (packet.data.len() * 8) - 1;
            while !((last_byte & 0x80) >= 1) {
                last_byte *= 2;
                bit_size -= 1;
            }
            self.received_packet(BitReader::new(&mut packet.data.as_slice(), bit_size))?;
            return Ok(())
        }
        Err(ErrorKind::ReplayParseError("malformed packet".to_string()).into())
    }

    #[inline]
    fn received_packet(&mut self, mut reader: BitReader) -> crate::Result<()> {
        self.packet_index += 1;
        while !reader.at_end() {
            let b_control = reader.read_bit()?;
            let mut bunch = DataBunch {
                packet_id: self.packet_index,
                b_open: if b_control { reader.read_bit()? } else { false },
                b_close: if b_control { reader.read_bit()? } else { false },
                ..DataBunch::default()
            };
            bunch.close_reason = if bunch.b_close { ChannelCloseReason::parse(reader.read_serialized_int(ChannelCloseReason::MAX as u32)?).ok_or_else(|| ErrorKind::ReplayParseError("Invalid channel close reason".to_string()))? } else { ChannelCloseReason::Destroyed };
            bunch.b_dormant = bunch.close_reason == ChannelCloseReason::Dormancy;
            bunch.b_is_replication_paused = reader.read_bit()?;
            bunch.b_is_reliable = reader.read_bit()?;
            bunch.ch_index = reader.read_int_packed()?;
            bunch.b_has_package_map_exports = reader.read_bit()?;
            bunch.b_has_must_be_mapped_guids = reader.read_bit()?;
            bunch.b_partial = reader.read_bit()?;
            if bunch.b_is_reliable {
                bunch.ch_seq = self.in_reliable + 1;
            } else if bunch.b_partial {
                bunch.ch_seq = self.packet_index;
            } else {
                bunch.ch_seq = 0;
            }
            bunch.b_partial_initial = if bunch.b_partial { reader.read_bit()? } else { false };
            bunch.b_partial_final = if bunch.b_partial { reader.read_bit()? } else { false };
            if bunch.b_is_reliable || bunch.b_open {
                bunch.ch_name = ChannelName::parse(reader.read_bit_fname()?);
            }
            let mut bunch_data_bits = reader.read_serialized_int((1024 * 2) * 8)?;
            //bunch.data = vec![0u8; (bunch_data_bits / 8) as usize];
            bunch.data_bit_size = bunch_data_bits as usize;
            bunch.data = reader.read_bits(&mut bunch_data_bits)?;
            self.parse_bunch(bunch)?;
        }
        Ok(())
    }

    #[inline]
    fn parse_bunch(&mut self, bunch: DataBunch) -> crate::Result<()> {
        //let reader = BitReader::new(&mut bunch.data.as_slice(), bunch.data_bit_size);
        let channel_exists = self.channels[bunch.ch_index as usize].is_some();
        if bunch.b_is_reliable && bunch.ch_seq <= self.in_reliable {
            return Ok(()) // packet already processed
        }
        if !channel_exists && !bunch.b_is_reliable {
            if !(bunch.b_open && (bunch.b_close || bunch.b_partial)) {
                return Ok(())
            }
        }
        if !channel_exists {
            self.channels[bunch.ch_index as usize] = Some(UChannel {
                name: bunch.ch_name,
                index: bunch.ch_index,
                actor: None,
            });
        }
        self.received_next_bunch(bunch);
        Ok(())
    }

    #[inline]
    fn received_next_bunch(&mut self, mut bunch: DataBunch) -> crate::Result<()> {
        if bunch.b_is_reliable {
            self.in_reliable = bunch.ch_seq;
        }
        if bunch.b_partial {
            if bunch.b_partial_initial {
                if self.partial_bunch.is_some() {
                    let partial_bunch = self.partial_bunch.as_ref().unwrap();
                    if !partial_bunch.b_partial_final {
                        if partial_bunch.b_is_reliable {
                            if bunch.b_is_reliable {
                                return Ok(())
                            }
                            return Ok(())
                        }
                    }
                    self.partial_bunch = Option::None;
                }
                self.partial_bunch = Some(bunch.clone());
                return Ok(())
            } else {
                let mut b_sequence_matches = false;
                if self.partial_bunch.is_some() {
                    let partial_bunch = self.partial_bunch.as_mut().unwrap();
                    let b_reliable_sequences_matches = bunch.ch_seq == partial_bunch.ch_seq + 1;
                    let b_unreliable_sequence_matches = b_reliable_sequences_matches || (bunch.ch_seq == partial_bunch.ch_seq);
                    b_sequence_matches = if partial_bunch.b_is_reliable { b_reliable_sequences_matches } else { b_unreliable_sequence_matches };
                    return if !partial_bunch.b_partial_final && b_sequence_matches && partial_bunch.b_is_reliable == bunch.b_is_reliable {
                        if !bunch.b_has_package_map_exports && bunch.data.len() > 0 {
                            partial_bunch.data.append(&mut bunch.data);
                            partial_bunch.data_bit_size += bunch.data_bit_size;
                        }
                        if !bunch.b_has_package_map_exports && !bunch.b_partial_final && (bunch.data_bit_size % 8 != 0) {
                            return Ok(()) // not byte aligned
                        }
                        partial_bunch.ch_seq = bunch.ch_seq;
                        if bunch.b_partial_final {
                            if bunch.b_has_package_map_exports {
                                return Ok(())
                            }
                            partial_bunch.b_partial_final = true;
                            partial_bunch.b_close = bunch.b_close;
                            partial_bunch.b_dormant = bunch.b_dormant;
                            partial_bunch.close_reason = bunch.close_reason;
                            partial_bunch.b_is_replication_paused = bunch.b_is_replication_paused;
                            partial_bunch.b_has_must_be_mapped_guids = bunch.b_has_must_be_mapped_guids;
                            let clone = partial_bunch.clone();
                            self.received_sequenced_bunch(clone);
                            return Ok(());
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
            }
        }
        self.received_sequenced_bunch(bunch);
        Ok(())
    }

    fn process_bunch(&mut self, bunch: &DataBunch, mut reader: BitReader) -> crate::Result<()>  {
        let mut channel = self.channels[bunch.ch_index as usize].as_mut().expect("???");
        if channel.actor.is_none() {
            if !bunch.b_open {
                return Ok(()) // actor channel without open packet
            }
            let mut in_actor = Actor {
                actor_net_guid: NetworkGUID::load_internal_object(&mut reader, false, 0)?.0, // todo look at exports perhaps?
                ..Default::default()
            };
            if reader.at_end() && in_actor.actor_net_guid.is_dynamic() {
                return Ok(())
            }
            if in_actor.actor_net_guid.is_dynamic() {
                in_actor.archetype = NetworkGUID::load_internal_object(&mut reader, false, 0)?.0;
                in_actor.level = NetworkGUID::load_internal_object(&mut reader, false, 0)?.0;
                in_actor.location = reader.read_conditionally_serialized_quantized_vector(FVector::default())?;
                if reader.read_bit()? {
                    in_actor.rotation = reader.read_rotation_short()?;
                } else {
                    in_actor.rotation = FRotator::default();
                }
                in_actor.scale = reader.read_conditionally_serialized_quantized_vector(FVector(1 as f32, 1 as f32, 1 as f32))?;
                in_actor.velocity = reader.read_conditionally_serialized_quantized_vector(FVector::default())?;
            }
            channel.actor = Some(in_actor);
            //todo channelopened + other thing
            unimplemented!();
        }
        unimplemented!();
        Ok(())
    }

    fn received_actor_bunch(&mut self, bunch: &DataBunch) -> crate::Result<()> {
        let mut slice = bunch.data.as_slice();
        let mut reader = BitReader::new(&mut slice, bunch.data_bit_size);
        if bunch.b_has_must_be_mapped_guids {
            let guids = reader.read_u16::<LE>()?;
            for x in 0..guids {
                reader.read_int_packed()?;
            }
        }
        self.process_bunch(bunch, reader);
        Ok(())
    }

    fn received_sequenced_bunch(&mut self, bunch: DataBunch) -> crate::Result<bool> {
        self.received_actor_bunch(&bunch);
        if bunch.b_close {
            self.channels[bunch.ch_index as usize] = None;
            //todo channelclose thing
            return Ok(true)
        }
        Ok(false)
    }

}