use std::env;

use tiltify::campaign::Donation;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::{fs::OpenOptions, io::BufWriter, sync::mpsc::UnboundedReceiver};

pub(crate) async fn file_writer(
    mut rx_writer: UnboundedReceiver<Donation>,
    filename: &str,
) -> tiltify::Result<()> {
    let file_path = env::temp_dir().join(&filename);
    println!("Logging donations to {}", &file_path.to_string_lossy());
    while let Some(message) = rx_writer.recv().await {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path)
            .await?;
        let mut file = BufWriter::with_capacity(4096, file);

        let data = match serde_json::to_vec(&message) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to serialize Message... {}", err);
                continue;
            }
        };

        file.seek(std::io::SeekFrom::End(0)).await?;
        file.write_all(&data).await?;
        file.write_all(b"\r\n").await?;
        file.flush().await?;
    }
    Ok(())
}
