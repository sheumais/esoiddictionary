use eso_skill_data::SkillIndexEntry;

#[derive(Clone, PartialEq)]
pub enum IndexState {
    Loading,
    Ready(Vec<SkillIndexEntry>),
    Failed(String),
}

/// Parse `index.bin` — format: [count: u32 BE][entry × count: 20 bytes each]
/// Each entry: [ability_id: u32 BE][start_offset: u64 BE][end_offset: u64 BE]
pub fn parse_index(bytes: &[u8]) -> Result<Vec<SkillIndexEntry>, String> {
    if bytes.len() < 4 {
        return Err("index.bin too short".into());
    }
    let count = u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let expected = 4 + count * 20;
    if bytes.len() < expected {
        return Err(format!(
            "index.bin truncated: expected {expected} bytes, got {}",
            bytes.len()
        ));
    }
    let entries = (0..count)
        .map(|i| {
            let o = 4 + i * 20;
            SkillIndexEntry::from_bytes(bytes[o..o + 20].try_into().unwrap())
        })
        .collect();
    Ok(entries)
}

/// Binary search a sorted index slice for `ability_id`.
pub fn find_entry(index: &[SkillIndexEntry], ability_id: u32) -> Option<SkillIndexEntry> {
    index
        .binary_search_by_key(&ability_id, |e| e.ability_id)
        .ok()
        .map(|i| index[i])
}