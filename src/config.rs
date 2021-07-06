use std::path::Path;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) access_token: String,
    pub(crate) campaign_id: String,
}


impl Config {
    pub(crate) async fn from_path(path: &Path) -> tiltify::Result<Self> {
        let contents = tokio::fs::read(path).await?;
        Ok(serde_json::from_slice(&contents)?)

    }
}