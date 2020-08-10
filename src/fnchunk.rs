use crate::uchunk::{EventChunk, HeaderChunk};
use std::io::{Read, Cursor};
use byteorder::ReadBytesExt;
use crate::ureplay::UReplay;
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub struct Elimination {
    pub victim_id: String,
    pub killer_id: String,
    pub gun_type: u8,
    pub knocked: bool
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct MatchStats {
    pub unknown: u32,
    pub accuracy: f32,
    pub assists: u32,
    pub eliminations: u32,
    pub weapon_damage: u32,
    pub other_damage: u32,
    pub revives: u32,
    pub damage_taken: u32,
    pub damage_to_structures: u32,
    pub materials_gathered: u32,
    pub materials_used: u32,
    pub total_travelled: u32
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct TeamStats {
    pub unknown: u32,
    pub position: u32,
    pub total_players: u32
}

impl Elimination {
    fn parse_player(cursor: &mut Cursor<Vec<u8>>) -> crate::Result<String> {
        let indicator = cursor.read_u8()?;
        return Ok(match indicator {
            0x03 => {
                "Bot".to_string()
            }
            0x10 => {
                let str: String = bincode::deserialize_from(cursor)?;
                str
            }
            _ => {
                let size = cursor.read_u8()?;
                let mut guid_bytes = vec![0u8; size as usize];
                cursor.read(guid_bytes.as_mut_slice());
                hex::encode(guid_bytes)
            }
        })
    }
    pub fn parse(e: EventChunk) -> crate::Result<Elimination> {
        //e.data.as_slice();
        if e.group != "playerElim" {
            return Err(crate::ErrorKind::ReplayParseError("tried to parse another chunk as elim chunk".to_string()).into());
        }
        let mut cursor = Cursor::new(e.data);
        cursor.read(&mut [0u8; 85]);
        return Ok(Elimination {
            victim_id: Elimination::parse_player(&mut cursor)?,
            killer_id: Elimination::parse_player(&mut cursor)?,
            gun_type: cursor.read_u8()?,
            knocked: cursor.read_u32::<byteorder::LE>()? != 0
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct FNSkim {
    pub header: HeaderChunk,
    pub team_stats: TeamStats,
    pub match_stats: MatchStats,
    pub eliminations: Vec<Elimination>
}

impl FNSkim {
    pub fn skim(replay: UReplay) -> crate::Result<FNSkim> {
        let mut skim = FNSkim::default();
        for x in replay.chunks {
            match x.variant {
                0 => {
                    skim.header = HeaderChunk::parse(x)?;
                }
                3 => {
                    let e_chunk = EventChunk::parse(x, replay.meta.encryption_key.as_slice())?;
                    if e_chunk.group == "playerElim" {
                        skim.eliminations.push(Elimination::parse(e_chunk)?);
                    } else {
                        match &*e_chunk.metadata {
                            "AthenaMatchStats" => {
                                skim.match_stats = bincode::deserialize(e_chunk.data.as_slice())?;
                            }
                            "AthenaTeamMatchStats" => {
                                skim.team_stats = bincode::deserialize(e_chunk.data.as_slice())?;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(skim)
    }
}