use crate::uetypes::{UEReadExt};
use byteorder::{ReadBytesExt, LE};
use std::io::Read;

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

#[derive(Debug, Default, PartialEq)]
pub struct DemoFrame {
    pub current_level_index: u32,
    pub time_seconds: f32,
    pub export_data: Vec<NetFieldExports>
}

impl DemoFrame {
    pub fn parse(cursor: &mut &[u8]) -> crate::Result<DemoFrame> {
        Ok(DemoFrame {
            current_level_index: cursor.read_u32::<LE>()?,
            time_seconds: cursor.read_f32::<LE>()?,
            export_data: NetFieldExports::parse(cursor)?,
        })
    }
}