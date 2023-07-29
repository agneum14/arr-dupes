# arr-dupes

Despite my configurations to the contrary, Radarr, Sonarr, and Lidarr sometimes don't remove torrents from my Transmission client, instead only removing the hard links in my media directories.
This program cross-references the inodes of files in the specified Transmission client download directories with those from the media directories, then connects with Transmission to prompt removal of the unmatched torrents.

## Installation

1. Clone the repo
2. Edit ```src/config.rs``` to suit your setup
3. Install with Cargo as you prefer

## Usage

When running the program, your Transmission password must be passed via the ```TPASS``` environment variable:

```TPASS="yourpassword" arr-dupes```
