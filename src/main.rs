mod config;
mod donation_cache;
mod file_writer;

use std::collections::HashSet;
use std::env::current_exe;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use tiltify::campaign::Donation;
use tiltify::client::TiltifyClient;
use tokio::fs::canonicalize;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::sleep;

use crate::config::Config;
use crate::donation_cache::get_seen_donataions;
use crate::file_writer::file_writer;

async fn base_dir() -> tiltify::Result<PathBuf> {
    let exe = current_exe()?;
    let mut exe = canonicalize(exe).await?;
    exe.pop();
    Ok(exe)
}

async fn get_new_donations(
    client: &TiltifyClient,
    campaign_id: &str,
    seen_donation_ids: &HashSet<u64>,
) -> tiltify::Result<Vec<Donation>> {
    let mut donations = Vec::new();

    let mut pager = client.campaign(campaign_id).donations().await?;
    loop {
        for donation in pager.data.drain(..) {
            if seen_donation_ids.contains(&donation.id) {
                break;
            }

            donations.push(donation)
        }

        pager = match pager.prev().await? {
            Some(pager) => pager,
            None => break,
        };
    }

    Ok(donations)
}

#[tokio::main]
async fn main() -> tiltify::Result<()> {
    let base_dir = base_dir().await?;
    let mut config_path = base_dir.clone();
    config_path.push("tiltify-donation-tailer.json");
    if !config_path.is_file() {
        return Err(anyhow!("Config file not found at {:?}", &config_path));
    }

    let config = Config::from_path(&config_path).await?;

    let client = TiltifyClient::new(&config.access_token)?;
    let filename = format!("tiltify-donations-{}.txt", &config.campaign_id);

    let mut seen_donation_ids = get_seen_donataions(&filename).await?;

    println!("Starting tailer for Campain {}", &config.campaign_id);
    let (tx_writer, rx_writer) = unbounded_channel::<Donation>();
    let writer_filename = filename.clone();
    tokio::spawn(async move {
        file_writer(rx_writer, &writer_filename).await?;
        Ok::<(), tiltify::Error>(())
    });

    loop {
        let new_donations =
            get_new_donations(&client, &config.campaign_id, &seen_donation_ids).await?;
        println!("Checking for new donations... {} found.", &new_donations.len());
        for new_donation in new_donations.into_iter().rev() {
            seen_donation_ids.insert(new_donation.id);
            println!(
                "Seen new donation: {}, {}, {}, {:?}",
                &new_donation.id, &new_donation.amount, &new_donation.name, &new_donation.comment,
            );
            tx_writer.send(new_donation)?;
        }
        sleep(Duration::from_millis(5000)).await;
    }
}
