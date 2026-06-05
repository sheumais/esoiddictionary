use std::{io::{Cursor, Read}, sync::OnceLock};
use eso_skill_data::SkillData34;
use ruzstd::decoding::StreamingDecoder;

const COMPRESSED: &[u8] = include_bytes!("../static/data.bin.zst");
static DATA: OnceLock<Vec<u8>> = OnceLock::new();

use crate::index_state::find_entry;

pub fn init_data() -> Result<(), String> {
    if DATA.get().is_some() {
        return Ok(());
    }

    let raw = Cursor::new(COMPRESSED);

    let mut decoder = StreamingDecoder::new(raw).map_err(|e| e.to_string())?;

    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| e.to_string())?;

    let _ = DATA.set(decompressed);

    Ok(())
}

pub fn read_bytes(range: Option<(u64, u64)>) -> Result<&'static [u8], String> {
    let data = DATA.get().ok_or("not initialized")?;

    match range {
        None => Ok(data.as_slice()),
        Some((start,end)) => {
            let s = start as usize;
            let e = end as usize;

            if e > data.len() {
                return Err("out of bounds".into());
            }

            Ok(&data[s..e])
        }
    }
}

pub fn get_skill(id: &u32) -> Option<SkillData34> {
    let index = find_entry(*id);
    match index {
        Some(i) => {
            let data = read_bytes(Some((i.start_offset, i.end_offset)));
            match data {
                Ok(a) => {
                    match SkillData34::from_bytes(a) {
                        Ok(record) => {return Some(record)}
                        Err(_) => return None,
                    }
                },
                Err(_) => return None,
            }
        },
        None => return None,
    }
}