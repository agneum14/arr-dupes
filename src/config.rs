use std::path::PathBuf;

pub const URL: &str = "http://127.0.0.1:9091/transmission/rpc";
pub const USER: &str = "username";

// prompt for confirmation before removing torrents
pub const CONFIRM: bool = true;
// avoid H&R (seconds)
pub const MIN_SEED_TIME: i64 = 604800;

pub fn download_dirs() -> Vec<PathBuf> {
    vec![
        path("/path/to/torrent/downloads/dir1"),
        path("/path/to/torrent/downloads/dir2"),
    ]
}

pub fn media_dirs() -> Vec<PathBuf> {
    vec![
        path("/path/to/media/dir1"),
        path("/path/to/media/dir2"),
        path("/path/to/media/dir3"),
    ]
}

// ignore, confirming your directories
fn path(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    match path_buf.exists() && path_buf.is_dir() && path_buf.is_absolute() {
        true => path_buf,
        false => panic!("\"{}\" isn't a valid, absolute directory", path),
    }
}
