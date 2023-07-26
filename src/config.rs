pub const URL: &str = "http://127.0.0.1:9091/transmission/rpc";
pub const USER: &str = "username";

// prompt for confirmation before removing torrents
pub const CONFIRM: bool = true;
// avoid H&R (seconds)
pub const MIN_SEED_TIME: i64 = 604800;

pub fn download_dirs() -> Vec<String> {
    vec![
        String::from("/path/to/torrent/downloads/dir1"),
        String::from("/path/to/torrent/downloads/dir2"),
    ]
}

pub fn media_dirs() -> Vec<String> {
    vec![
        String::from("/path/to/media/dir1"),
        String::from("/path/to/media/dir2"),
        String::from("/path/to/media/dir3"),
    ]
}
