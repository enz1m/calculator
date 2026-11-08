use crate::app::Saved;
use enzui_core::atomic_state::{open_state_file_nofollow, save_atomic_raw};
use std::io::{self, Read};
use std::path::Path;

const MAX_BYTES: u64 = 262_144;

pub(crate) fn load(path: &Path) -> io::Result<Saved> {
    let file = match open_state_file_nofollow(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Saved::default()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "history exceeds size limit",
        ));
    }
    let saved: Saved = serde_json::from_slice(&bytes)?;
    if saved.history.len() > 100
        || saved.history.iter().any(|entry| {
            entry.expression.len() > 1024 || entry.result.len() > 128 || !entry.answer.is_finite()
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid history entry",
        ));
    }
    Ok(saved)
}

pub(crate) fn save(path: &Path, saved: &Saved) -> io::Result<()> {
    let bytes = serde_json::to_vec(saved)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "history exceeds size limit",
        ));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    save_atomic_raw(path, &bytes)
}
