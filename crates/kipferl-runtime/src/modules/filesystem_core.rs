use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn unique_path(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(format!("{prefix}{}-{stamp}-{counter}", std::process::id()))
}

pub(super) fn temporary_directory() -> PathBuf {
    std::env::var_os("TMPDIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub(super) fn path_string(path: &Path) -> Option<String> {
    path.to_str().map(ToOwned::to_owned)
}
