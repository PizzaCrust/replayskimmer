use crate::uetypes::{UEReadExt};
use byteorder::{ReadBytesExt, LE};
use std::io::Read;
use serde::Deserialize;
use serde::export::fmt::Debug;
use serde::export::Formatter;

#[derive(Debug, PartialEq)]
pub struct NetFieldExport { //check if exported before deserialization!
    pub handle: u32,
    pub compatible_checksum: u32,
    pub name: String //FNAME!!
}

impl NetFieldExport {
    pub fn parse(mut cursor: &mut &[u8]) -> crate::Result<Option<NetFieldExport>> {
        let is_exported = cursor.read_u8()? != 0;
        if is_exported {
            return Ok(Some(NetFieldExport {
                handle: cursor.read_int_packed()?,
                compatible_checksum: cursor.read_u32::<LE>()?,
                name: cursor.read_fname()?,
            }))
        }
        Ok(None)
    }
}

// varint land as we are in networking territory :)
#[derive(Debug, Default, PartialEq)]
pub struct NetFieldExports {
    pub path_name_index: u32,
    pub is_exported: bool,

    // IF EXPORTED
    pub path_name: Option<String>, // normal ue string
    pub num_exports: Option<u32>,
    //END

    pub export: Option<NetFieldExport>
}

impl NetFieldExports {
    pub fn parse(mut cursor: &mut &[u8]) -> crate::Result<Vec<NetFieldExports>> {
        let num_layout_cmd_exports = cursor.read_int_packed()?;
        let mut exports_vec: Vec<NetFieldExports> = Vec::new();
        for _ in 0..num_layout_cmd_exports {
            let mut export = NetFieldExports {
                path_name_index: cursor.read_int_packed()?,
                is_exported: cursor.read_int_packed()? != 0,
                ..Default::default()
            };
            if export.is_exported {
                export.path_name = Some(cursor.read_fstring()?);
                export.num_exports = Some(cursor.read_int_packed()?)
            }
            export.export = NetFieldExport::parse(cursor)?;
            exports_vec.push(export);
        }
        Ok(exports_vec)
    }
}

#[derive(Debug, Default)]
pub struct DemoFrame {
    pub current_level_index: u32,
    pub time_seconds: f32,
    pub export_data: Vec<NetFieldExports>,
    pub packets: Vec<PlaybackPacket>
}

#[derive(PartialEq)]
pub enum PacketState {
    Success, End
}

impl Default for PacketState {
    fn default() -> Self {
        return PacketState::End;
    }
}

pub struct PlaybackPacket {
    state: PacketState,
    data: Vec<u8>
}

impl Debug for PlaybackPacket {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&*format!("playback packet with size of {}", self.data.len()))
    }
}

impl DemoFrame {
    pub fn parse(cursor: &mut &[u8]) -> crate::Result<DemoFrame> {
        let mut frame = DemoFrame {
            current_level_index: cursor.read_u32::<LE>()?,
            time_seconds: cursor.read_f32::<LE>()?,
            ..Default::default()
        };
        frame.export_data = NetFieldExports::parse(cursor)?;
        // net guid here but fuck that data
        let num_guids = cursor.read_int_packed()?;
        for _ in 0..num_guids {
            let size = cursor.read_u32::<LE>()?;
            cursor.read(vec![0u8; size as usize].as_mut_slice());
            // todo ^ maybe change all of these skips to write to buffer to positional skips to increase performance?
        }
        let num_streaming_levels = cursor.read_int_packed()?;
        for _ in 0..num_streaming_levels {
            cursor.read_fstring()?; // ok this is so inefficient
        }
        cursor.read_u64::<LE>()?; // external offset
        loop {
            let external_data_num_bits = cursor.read_int_packed()?;
            if external_data_num_bits == 0 {
                break;
            }
            let net_guid = cursor.read_int_packed()?;
            let bytes = (external_data_num_bits + 7) >> 3;
            cursor.read(vec![0u8; bytes as usize].as_mut_slice());
        }
        let game_specific_data_size = cursor.read_u64::<LE>()?;
        cursor.read(vec![0u8; game_specific_data_size as usize].as_mut_slice())?;
        loop {
            cursor.read_int_packed()?; // seen level index
            let size = cursor.read_u32::<LE>()?;
            let mut packet: PlaybackPacket = PlaybackPacket {
                state: if size == 0 || size < 0 { PacketState::End } else { PacketState::Success },
                data: vec![]
            };
            if let PacketState::Success = packet.state {
                packet.data = vec![0u8; size as usize];
                cursor.read(packet.data.as_mut_slice())?;
            } else {
                break;
            }
            frame.packets.push(packet);
        }
        Ok(frame)
    }
}