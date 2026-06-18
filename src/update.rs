use serde::Deserialize;

const REPO_OWNER: &str = "davelet";
const REPO_NAME: &str = "xiaohe-yinxing-dict";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    html_url: String,
}

pub fn check_for_update(current_version: &str) -> Option<UpdateInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("xiaohe-yinxing-dict")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }

    let release: GitHubRelease = response.json().ok()?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    if !is_newer_version(current_version, &latest_version) {
        return None;
    }

    Some(UpdateInfo {
        latest_version,
        download_url: release.html_url,
        release_notes: release.body,
    })
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("1.0", "1.0.1"));
        assert!(!is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("1.1.0", "1.0.0"));
        assert!(!is_newer_version("2.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0"));
        assert!(!is_newer_version("1.0.0", "1"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }
}
