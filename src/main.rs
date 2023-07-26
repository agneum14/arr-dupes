use std::collections::HashSet;
use std::io::{stdin, stdout, Write};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::str::FromStr;
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
    let pass = env::var("TPASS").unwrap();
    let basic_auth = BasicAuth {
        user: String::from(config::USER),
        password: String::from(pass),
    };
    let client = TransClient::with_auth((&config::URL).parse().unwrap(), basic_auth);

    client
}

fn add_media_inodes(inodes: &mut HashSet<u64>, media_dir: &String) {
    println!("adding inodes for {}", media_dir);

    for entry in WalkDir::new(media_dir) {
        let entry = entry.unwrap();

        if !(entry.file_type().is_file()) {
            continue;
        }

        let meta = fs::metadata(entry.path()).unwrap();
        let inode = meta.ino();
        inodes.insert(inode);
    }
}

async fn add_torrents(
    client: &mut TransClient,
    torrents: &mut Vec<TorrentData>,
    download_dir: &String,
) {
    println!("fetching torrents with download_dir {}", download_dir);

    let path_start = PathBuf::from_str(download_dir).unwrap();
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
        .unwrap();

    for torrent in res.arguments.torrents.into_iter().filter(|t| {
        t.error_string.as_ref().unwrap() == ""
            && t.download_dir.as_ref().unwrap() == download_dir
            && t.seconds_seeding.as_ref().unwrap() >= &config::MIN_SEED_TIME
    }) {
        let mut inodes: Vec<u64> = Vec::new();

        let name = torrent.name.as_ref().unwrap().to_owned();
        let id = torrent.id.as_ref().unwrap().to_owned();

        for file in torrent.files.unwrap() {
            let path_end = PathBuf::from_str(&file.name).unwrap();
            let path = path_start.join(path_end);
            let meta = fs::metadata(path).unwrap();

            if meta.is_file() {
                inodes.push(meta.ino());
            }
        }

        torrents.push(TorrentData { name, id, inodes });
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
    let mut media_inodes: HashSet<u64> = HashSet::new();
    let mut torrent_data: Vec<TorrentData> = Vec::new();
    let mut client = auth();

    for media_dir in config::media_dirs() {
        add_media_inodes(&mut media_inodes, &media_dir);
    }

    for download_dir in config::download_dirs() {
        add_torrents(&mut client, &mut torrent_data, &download_dir).await;
    }

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
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        if input != "y\n" && input != "Y\n" && input != "\n" {
            return;
        }
        println!();
    }

    for td in &torrent_data {
        remove_torrent(&mut client, &td).await;
    }
}
