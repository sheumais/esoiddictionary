use std::collections::HashMap;
use std::sync::OnceLock;

use eso_skill_data::SkillIndexEntry;

static INDEX_BYTES: &[u8] = include_bytes!("../static/index.bin");

pub static INDEX_MAP: OnceLock<HashMap<u32, SkillIndexEntry>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
pub enum IndexState {
    Loading,
    Failed(String),
    Ready,
}

pub fn parse_index(bytes: &[u8]) -> Result<Vec<SkillIndexEntry>, String> {
    if bytes.len() < 4 {
        return Err("index.bin too short".into());
    }

    let count = u32::from_be_bytes(
        bytes[0..4]
            .try_into()
            .map_err(|_| "failed to read count")?,
    ) as usize;

    let expected = 4 + count * 20;
    if bytes.len() < expected {
        return Err(format!(
            "index.bin truncated: expected {expected} bytes, got {}",
            bytes.len()
        ));
    }

    let mut entries = Vec::with_capacity(count);

    for i in 0..count {
        let offset = 4 + i * 20;

        let chunk: [u8; 20] = bytes[offset..offset + 20]
            .try_into()
            .map_err(|_| "invalid entry slice")?;

        let entry = SkillIndexEntry::from_bytes(&chunk);
        entries.push(entry);
    }

    Ok(entries)
}

pub fn init_index_cache() -> Result<(), String> {
    if INDEX_MAP.get().is_some() {
        return Ok(());
    }

    let entries = parse_index(INDEX_BYTES)?;

    let mut map = HashMap::with_capacity(entries.len());
    for entry in entries {
        map.insert(entry.ability_id, entry);
    }

    INDEX_MAP
        .set(map)
        .map_err(|_| "index map already initialized".to_string())?;

    Ok(())
}

pub fn find_entry(ability_id: u32) -> Option<&'static SkillIndexEntry> {
    let map = INDEX_MAP.get()?;
    map.get(&ability_id)
}