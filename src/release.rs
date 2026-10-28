use crate::self_update::UpdateError;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

impl Release {
    pub fn parse(json: &str) -> Result<Self, UpdateError> {
        serde_json::from_str(json).map_err(|e| UpdateError::InvalidRelease(e.to_string()))
    }

    pub fn asset(&self, name: &str) -> Result<&Asset, UpdateError> {
        self.assets
            .iter()
            .find(|asset| asset.name == name)
            .ok_or_else(|| UpdateError::MissingAsset(name.to_string()))
    }
}
