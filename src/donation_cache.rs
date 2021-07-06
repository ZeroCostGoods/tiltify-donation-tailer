use std::collections::HashSet;
use std::env;

use tiltify::campaign::Donation;
use tokio::fs::OpenOptions;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

pub(crate) async fn get_seen_donataions(filename: &str) -> tiltify::Result<HashSet<u64>> {
    let file_path = env::temp_dir().join(&filename);
    let mut seen_donations: HashSet<u64> = HashSet::new();
    if !file_path.is_file() {
        return Ok(seen_donations);
    }

    let file = OpenOptions::new().read(true).open(file_path).await?;
    let file = BufReader::with_capacity(4096, file);

    let mut lines = file.lines();
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            println!("Skipping empty line...")
        }
        let donation: Donation = match serde_json::from_str(&line) {
            Ok(donation) => donation,
            Err(err) => {
                println!("Failed to read line: {}", err);
                continue;
            }
        };
        seen_donations.insert(donation.id);
    }

    Ok(seen_donations)
}
