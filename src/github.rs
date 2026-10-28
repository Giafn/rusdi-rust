use crate::{
    platform::Target,
    release::{Asset, Release},
    self_update::UpdateError,
};

pub const OWNER: &str = "Giafn";
pub const REPOSITORY: &str = "rusdi-rust";

pub fn latest_release() -> Result<Release, UpdateError> {
    let url = format!("https://api.github.com/repos/{OWNER}/{REPOSITORY}/releases/latest");
    let response = ureq::get(&url)
        .header("User-Agent", "rudis-lite")
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    Release::parse(&body)
}

pub fn release_assets(release: &Release, target: Target) -> Result<(&Asset, &Asset), UpdateError> {
    Ok((
        release.asset(target.asset_name)?,
        release.asset("SHA256SUMS")?,
    ))
}
