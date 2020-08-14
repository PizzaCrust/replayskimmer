use crate::uetypes::{UEReadExt};
use byteorder::{ReadBytesExt, LE};
use std::io::Read;
use serde::Deserialize;
use serde::export::fmt::Debug;
use serde::export::Formatter;
use crate::data::DataChunk;
use crate::ErrorKind;
use std::collections::HashMap;
use crate::data::packet::PacketParser;

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
    //pub net_guid_val_to_path: HashMap<u32, String>, todo return in the future
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
    pub state: PacketState,
    pub data: Vec<u8>
}

impl Debug for PlaybackPacket {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&*format!("playback packet with size of {}", self.data.len()))
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct NetworkGUID(pub u32);

pub trait StringExt {
    fn remove_all_path_prefixes(self) -> String;
    fn remove_path_prefix(self, to_remove: String) -> String;
    fn clean_path_suffix(self) -> String;
}

impl StringExt for String {
    fn remove_all_path_prefixes(self) -> String {
        for (index, x) in self.char_indices().rev() {
            match x {
                '.' => {
                    return (&self[(index + 1)..self.len()]).to_owned()
                }
                '/' => {
                    return self;
                }
                _ => {}
            }
        }
        self.remove_path_prefix("Default__".to_string())
    }

    fn remove_path_prefix(self, to_remove: String) -> String {
        if to_remove.len() > self.len() {
            return self;
        }
        self.strip_prefix(&*to_remove).unwrap_or_else(|| &*self).to_string()
    }

    fn clean_path_suffix(self) -> String {
        for (i, x) in self.char_indices().rev() {
            if !x.is_numeric() && x != '_' {
                return (&self[0..(i + 1)]).to_owned()
            }
        }
        self
    }
}

impl NetworkGUID {
    #[inline]
    pub fn is_valid(&self) -> bool { self.0 > 0 }
    #[inline]
    pub fn is_default(&self) -> bool { self.0 == 1 }
    #[inline]
    pub fn is_dynamic(&self) -> bool { self.0 > 0 && (self.0 & 1) != 1 }
    // returns network guid + (net guid value, path name)
    pub(crate) fn load_internal_object<T: Read>(cursor: &mut T,
                                is_exporting_net_guid_bunch: bool,
                                load_object_recursion_counter: i32) -> crate::Result<(NetworkGUID, Option<(NetworkGUID, String)>)> {
        if load_object_recursion_counter > 16 {
            //return Err(ErrorKind::ReplayParseError("Hit recursion limit".to_string()).into());
            return Ok((NetworkGUID::default(), None));
        }
        let guid = NetworkGUID(cursor.read_int_packed()?);
        if !guid.is_valid() {
            return Ok((guid, None));
        }
        if guid.is_default() || is_exporting_net_guid_bunch {
            let flags = cursor.read_u8()?;
            if (flags & 1) != 0 { //bHasPath
                let outer_guid = Self::load_internal_object(cursor, true, load_object_recursion_counter + 1)?;
                let path_name = cursor.read_fstring()?;
                if (flags & 4) != 0 { //bHasNetworkChecksum
                    cursor.read_u32::<LE>()?; //network checksum
                }
                let set = Some((guid.clone(), path_name.remove_all_path_prefixes()));
                if is_exporting_net_guid_bunch {
                    return Ok((guid, set));
                }
            }
        }
        Ok((guid, None))
    }
}

impl DemoFrame {
    pub fn parse(cursor: &mut &[u8], packet_parser: &mut PacketParser) -> crate::Result<DemoFrame> {
        let mut frame = DemoFrame {
            current_level_index: cursor.read_u32::<LE>()?,
            time_seconds: cursor.read_f32::<LE>()?,
            ..Default::default()
        };
        frame.export_data = NetFieldExports::parse(cursor)?;
        let num_guids = cursor.read_int_packed()?;
        for _ in 0..num_guids {
            let size = cursor.read_i32::<LE>()?;
            let mut uobject = vec![0u8; size as usize];
            cursor.read(uobject.as_mut_slice())?;
            let o = NetworkGUID::load_internal_object(&mut uobject.as_slice(), true, 0)?;
            if let Some((key, value)) = o.1 {
                //frame.net_guid_val_to_path.insert(key, value);
                packet_parser.net_guid_cache.net_guid_to_path.insert(key, value);
            }
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
                frame.packets.push(packet);
                break;
            }
            packet_parser.received_raw_packet(&packet)?;
            frame.packets.push(packet);
        }
        Ok(frame)
    }
    pub fn parse_data(data_chunk: DataChunk, packet_parser: &mut PacketParser) -> crate::Result<Vec<DemoFrame>> {
        let mut slice = data_chunk.data.as_slice();
        let mut demo_frames: Vec<DemoFrame> = Vec::new();
        while !slice.is_empty() {
            demo_frames.push(Self::parse(&mut slice, packet_parser)?);
        }
        Ok(demo_frames)
    }
}