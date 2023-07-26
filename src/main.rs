use std::collections::HashSet;
use std::io::{stdin, stdout, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::{env, fs, vec};

use transmission_rpc::types::{BasicAuth, Id, TorrentGetField};
use transmission_rpc::TransClient;
use walkdir::WalkDir;

mod config;

struct TorrentData {
    name: String,
    id: i64,
    inodes: Vec<u64>,
}

impl TorrentData {
    fn unmatched(&self, media_inodes: &HashSet<u64>) -> bool {
        for inode in &self.inodes {
            if media_inodes.contains(&inode) {
                return false;
            }
        }

        true
    }
}

fn auth() -> TransClient {
    let pass = env::var("TPASS").expect("set TPASS environment variable");
    let basic_auth = BasicAuth {
        user: String::from(config::USER),
        password: String::from(pass),
    };
    let url = (&config::URL).parse().expect("can't parse URL");
    let client = TransClient::with_auth(url, basic_auth);

    client
}

fn add_media_inodes(inodes: &mut HashSet<u64>, media_dir: &PathBuf) {
    println!("adding inodes for {}", media_dir.display());

    for entry in WalkDir::new(media_dir) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("WARNING: can't get entry: {}", err);
                continue;
            }
        };

        if !(entry.file_type().is_file()) {
            continue;
        }

        let meta = match fs::metadata(entry.path()) {
            Ok(meta) => meta,
            Err(err) => {
                println!(
                    "WARNING: can't get inode for file \"{}\": {}",
                    entry.path().display(),
                    err
                );
                continue;
            }
        };
        let inode = meta.ino();
        inodes.insert(inode);
    }
}

async fn add_torrents(
    client: &mut TransClient,
    torrent_data: &mut Vec<TorrentData>,
    download_dirs: &Vec<PathBuf>,
) {
    println!("fetching torrents");

    let res = client
        .torrent_get(
            Some(vec![
                TorrentGetField::DownloadDir,
                TorrentGetField::Name,
                TorrentGetField::Id,
                TorrentGetField::Files,
                TorrentGetField::LeftUntilDone,
                TorrentGetField::ErrorString,
                TorrentGetField::SecondsSeeding,
            ]),
            None,
        )
        .await
        .expect("can't connect to Transmission");

    for t in res.arguments.torrents {
        let err_str = t.error_string.unwrap_or("z".to_string());
        let download_dir = t.download_dir.unwrap_or("".to_string());
        let download_dir = PathBuf::from(&download_dir);
        let sec_seed = t
            .seconds_seeding
            .as_ref()
            .unwrap_or(&(config::MIN_SEED_TIME + 1));

        if !(err_str == ""
            && download_dirs.contains(&download_dir)
            && config::MIN_SEED_TIME >= *sec_seed)
        {
            continue;
        }

        let mut inodes: Vec<u64> = Vec::new();

        let name = t.name.unwrap_or("UNKNOWN".to_string());
        let id = match t.id {
            Some(id) => id,
            None => {
                println!(
                    "WARNING: can't get ID for torrent \"{}\", omitting...",
                    name
                );
                continue;
            }
        };

        let files = match t.files.as_ref() {
            Some(files) => files,
            None => {
                println!("WARNING: no files for torrent \"{}\", omitting...", name);
                continue;
            }
        };
        for file in files {
            let path_end = Path::new(&file.name);
            let path = (&download_dir).join(path_end);
            let meta = match fs::metadata(&path) {
                Ok(meta) => meta,
                Err(err) => {
                    println!(
                        "WARNING: can't get inode for file \"{}\" for torrent \"{}\": {}, omitting...",
                        path.display(),
                        name,
                        err
                    );
                    continue;
                }
            };

            if meta.is_file() {
                inodes.push(meta.ino());
            }
        }

        torrent_data.push(TorrentData { name, id, inodes });
    }
}

async fn remove_torrent(client: &mut TransClient, td: &TorrentData) {
    println!("removing {}", td.name);

    let res = client.torrent_remove(vec![Id::Id(td.id)], true).await;
    match res {
        Ok(_) => println!("success"),
        Err(e) => println!("WARNING: failed to remove torrent: {}", e),
    }
}

#[tokio::main]
async fn main() {
    let download_dirs = config::download_dirs();
    let mut media_inodes: HashSet<u64> = HashSet::new();
    let mut torrent_data: Vec<TorrentData> = Vec::new();
    let mut client = auth();

    for media_dir in config::media_dirs() {
        add_media_inodes(&mut media_inodes, &media_dir);
    }

    add_torrents(&mut client, &mut torrent_data, &download_dirs).await;
    torrent_data.retain(|td| td.unmatched(&media_inodes));

    if torrent_data.len() == 0 {
        println!("No duplicates found.");
        return;
    }

    for td in &torrent_data {
        println!("{}", td.name);
    }

    if config::CONFIRM {
        let mut input = String::new();
        print!("The above torrents have been marked for deletion. Continue? (Y/n): ");
        stdout().flush().unwrap_or_else(|_| println!());
        stdin().read_line(&mut input).expect("couln't read input");
        if input != "y\n" && input != "Y\n" && input != "\n" {
            return;
        }
        println!();
    }

    for td in &torrent_data {
        remove_torrent(&mut client, &td).await;
    }
}
