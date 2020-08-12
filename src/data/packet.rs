use crate::data::net::PlaybackPacket;
use crate::ErrorKind;
use std::io::Read;
use crate::uetypes::{ChannelName, ChannelCloseReason, UEReadExt, UnrealName};
use byteorder::{ReadBytesExt, LE};
use crate::strum::AsStaticRef;
use crate::data::BitReader;

#[derive(Default, Debug)]
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
    data: Vec<u8>
}

#[derive(Default, Copy, Clone)]
struct UChannel {
    name: ChannelName,
    index: u32
}

pub struct PacketParser {
    packet_index: i32, // 0
    in_reliable: i32, // 0
    channels: [UChannel; 32767],
}

impl PacketParser {
    pub fn new() -> PacketParser {
        PacketParser {
            packet_index: 0,
            in_reliable: 0,
            channels: [UChannel::default(); 32767]
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
            self.received_packet(BitReader::new(packet.data.as_slice(), bit_size))?;
            return Ok(())
        }
        Err(ErrorKind::ReplayParseError("malformed packet".to_string()).into())
    }

    #[inline]
    fn received_packet(&mut self, mut reader: BitReader) -> crate::Result<()> {
        self.packet_index += 1;
        while !reader.at_end() {
            let b_control = reader.read_bit();
            let mut bunch = DataBunch {
                packet_id: self.packet_index,
                b_open: if b_control { reader.read_bit() } else { false },
                b_close: if b_control { reader.read_bit() } else { false },
                ..DataBunch::default()
            };
            bunch.close_reason = if bunch.b_close { ChannelCloseReason::parse(reader.read_serialized_int(ChannelCloseReason::MAX as u32)).ok_or_else(|| ErrorKind::ReplayParseError("Invalid channel close reason".to_string()))? } else { ChannelCloseReason::Destroyed };
            bunch.b_dormant = bunch.close_reason == ChannelCloseReason::Dormancy;
            bunch.b_is_replication_paused = reader.read_bit();
            bunch.b_is_reliable = reader.read_bit();
            bunch.ch_index = reader.read_int_packed()?;
            bunch.b_has_package_map_exports = reader.read_bit();
            bunch.b_has_must_be_mapped_guids = reader.read_bit();
            bunch.b_partial = reader.read_bit();
            if bunch.b_is_reliable {
                bunch.ch_seq = self.in_reliable + 1;
            } else if bunch.b_partial {
                bunch.ch_seq = self.packet_index;
            } else {
                bunch.ch_seq = 0;
            }
            bunch.b_partial_initial = if bunch.b_partial { reader.read_bit() } else { false };
            bunch.b_partial_final = if bunch.b_partial { reader.read_bit() } else { false };
            if bunch.b_is_reliable || bunch.b_open {
                bunch.ch_name = ChannelName::parse(reader.read_bit_fname()?);
            }
            let bunch_data_bits = reader.read_serialized_int((1024 * 2) * 8);
            bunch.data = vec![0u8; (bunch_data_bits / 8) as usize];
            reader.read(bunch.data.as_mut_slice());
            break;
        }
        Ok(())
    }
}