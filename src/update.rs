use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use sha2::{Digest, Sha256};

const REPO_OWNER: &str = "davelet";
const REPO_NAME: &str = "xiaohe-yinxing-dict";

/// 代理获取策略：环境变量优先，其次读取系统代理
fn get_proxy_url() -> Option<String> {
    // 优先使用环境变量代理
    for var in &[
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        if let Ok(val) = std::env::var(var)
            && !val.is_empty()
        {
            return Some(val);
        }
    }
    // 回退到系统代理
    if let Ok(proxy) = sysproxy::Sysproxy::get_system_proxy()
        && proxy.enable
    {
        let url = format!("http://{}:{}", proxy.host, proxy.port);
        return Some(url);
    }
    None
}

fn http_client_builder() -> reqwest::blocking::ClientBuilder {
    let mut builder = reqwest::blocking::Client::builder().user_agent("xiaohe-yinxing-dict");
    if let Some(proxy_url) = get_proxy_url()
        && let Ok(proxy) = reqwest::Proxy::all(&proxy_url)
    {
        builder = builder.proxy(proxy);
    }
    builder
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub sha256: Option<String>,
    pub release_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Downloading,
    Installing,
    Done(PathBuf),
    Failed(String),
}

/// 详细更新进度（跨线程共享）
#[derive(Debug, Clone)]
pub struct UpdateProgress {
    pub message: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub sha256_ok: Option<bool>, // None=未校验, Some(true)=通过, Some(false)=不匹配
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    #[allow(dead_code)]
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check_for_update(current_version: &str) -> Option<UpdateInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = http_client_builder()
        .timeout(std::time::Duration::from_secs(15))
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

    let download_url = find_download_url(&release)?;
    let sha256 = find_checksum_url(&release).and_then(|url| fetch_checksum(&client, &url).ok());
    let release_notes = release.body;

    Some(UpdateInfo {
        latest_version,
        download_url,
        sha256,
        release_notes,
    })
}

fn find_download_url(release: &GitHubRelease) -> Option<String> {
    let suffix = platform_asset_suffix()?;
    release
        .assets
        .iter()
        .find(|a| a.name.ends_with(&suffix))
        .map(|a| a.browser_download_url.clone())
}

fn find_checksum_url(release: &GitHubRelease) -> Option<String> {
    let suffix = platform_asset_suffix()?;
    let checksum_name = format!("{}.sha256", suffix);
    release
        .assets
        .iter()
        .find(|a| a.name.ends_with(&checksum_name))
        .map(|a| a.browser_download_url.clone())
}

fn fetch_checksum(client: &reqwest::blocking::Client, url: &str) -> Result<String, String> {
    let resp = client.get(url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let text = resp.text().map_err(|e| e.to_string())?;
    let hash = text.split_whitespace().next().unwrap_or("").to_string();
    if hash.is_empty() {
        return Err("checksum file is empty".to_string());
    }
    Ok(hash)
}

fn platform_asset_suffix() -> Option<&'static str> {
    #[cfg(target_os = "macos")]
    {
        let arch = std::env::consts::ARCH;
        match arch {
            "aarch64" => Some("-macos-aarch64.zip"),
            "x86_64" => Some("-macos-x86_64.zip"),
            _ => None,
        }
    }
    #[cfg(target_os = "windows")]
    {
        Some("-windows.zip")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

pub fn download_update(
    info: &UpdateInfo,
    progress: &Arc<Mutex<UpdateProgress>>,
    cancelled: &AtomicBool,
) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir();
    let filename = info.download_url.rsplit('/').next().unwrap_or("update.zip");
    let zip_path = tmp_dir.join(filename);

    {
        let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
        p.message = "开始下载...".to_string();
        p.bytes_downloaded = 0;
        p.bytes_total = 0;
        p.sha256_ok = None;
    }

    let result = do_download(info, &zip_path, progress, cancelled);
    if result.is_err() {
        let _ = std::fs::remove_file(&zip_path);
    }
    let zip_path = result?;

    // 安装前检查取消
    if cancelled.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(&zip_path);
        return Err("已取消".to_string());
    }

    // 更新进度：SHA256 校验阶段
    if info.sha256.is_some() {
        {
            let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
            p.message = "正在验证 SHA256 完整性...".to_string();
        }
    }

    if let Some(ref expected) = info.sha256
        && let Err(e) = verify_checksum(&zip_path, expected)
    {
        {
            let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
            p.sha256_ok = Some(false);
            p.message = format!("SHA256 校验失败: {}", e);
        }
        let _ = std::fs::remove_file(&zip_path);
        return Err(e);
    }

    if info.sha256.is_some() {
        let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
        p.sha256_ok = Some(true);
        p.message = "SHA256 校验通过 ✓".to_string();
    }

    Ok(zip_path)
}

fn do_download(
    info: &UpdateInfo,
    zip_path: &Path,
    progress: &Arc<Mutex<UpdateProgress>>,
    cancelled: &AtomicBool,
) -> Result<PathBuf, String> {
    let client = http_client_builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut last_err = String::new();
    let mut response = None;
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        // 重试前检查取消，避免用户取消后仍在重试等待
        if cancelled.load(Ordering::Relaxed) {
            return Err("已取消".to_string());
        }
        match client.get(&info.download_url).send() {
            Ok(r) if r.status().is_success() => {
                response = Some(r);
                break;
            }
            Ok(r) => {
                let status = r.status();
                last_err = format!("HTTP {}", status);
                // 4xx 客户端错误重试无意义，直接失败
                if status.is_client_error() {
                    return Err(format!("下载失败: {}", last_err));
                }
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }
    let mut response =
        response.ok_or_else(|| format!("下载请求失败: 重试3次均失败 ({})", last_err))?;

    let content_length = response.content_length().unwrap_or(0);

    let mut file =
        std::fs::File::create(zip_path).map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut buffer = vec![0u8; 8192];

    {
        let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
        p.bytes_total = content_length;
    }

    loop {
        // 检查取消
        if cancelled.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(zip_path);
            return Err("已取消".to_string());
        }

        let bytes_read = response
            .read(&mut buffer)
            .map_err(|e| format!("读取下载数据失败: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..bytes_read])
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += bytes_read as u64;

        // 每 256KB 更新一次进度（避免锁竞争太频繁）
        if downloaded % 262_144 < bytes_read as u64
            && let Ok(mut p) = progress.lock()
        {
            p.bytes_downloaded = downloaded;
            if content_length > 0 {
                let ratio = downloaded as f64 / content_length as f64;
                let downloaded_mb = downloaded as f64 / 1_048_576.0;
                let total_mb = content_length as f64 / 1_048_576.0;
                p.message = format!(
                    "正在下载  {:.1}MB / {:.1}MB  ({:.0}%)",
                    downloaded_mb,
                    total_mb,
                    ratio * 100.0
                );
            } else {
                let downloaded_mb = downloaded as f64 / 1_048_576.0;
                p.message = format!("正在下载  {:.1}MB", downloaded_mb);
            }
        }
    }

    // 下载完成后再次检查取消
    if cancelled.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(zip_path);
        return Err("已取消".to_string());
    }

    drop(file);

    if downloaded < 1_000_000 {
        return Err("下载文件过小，可能不完整".to_string());
    }

    if content_length > 0 && downloaded != content_length {
        return Err(format!(
            "文件大小不一致: 预期 {} 字节，实际 {} 字节",
            content_length, downloaded
        ));
    }

    Ok(zip_path.to_path_buf())
}

fn verify_checksum(file_path: &Path, expected_sha256: &str) -> Result<(), String> {
    let contents = std::fs::read(file_path).map_err(|e| format!("读取文件失败: {}", e))?;
    let digest = Sha256::digest(&contents);
    let hex = format!("{:x}", digest);
    if hex != expected_sha256 {
        return Err(format!("校验失败: 预期 {}，实际 {}", expected_sha256, hex));
    }
    Ok(())
}

pub fn apply_update(zip_path: &Path) -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        apply_update_macos(zip_path)
    }
    #[cfg(target_os = "windows")]
    {
        apply_update_windows(zip_path)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = zip_path;
        Err("当前平台不支持自动更新".to_string())
    }
}

#[cfg(target_os = "macos")]
fn apply_update_macos(zip_path: &Path) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir();
    let extract_dir = tmp_dir.join(format!("xiaohe-update-{}", std::process::id()));
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {}", e))?;

    let zip_file =
        std::fs::File::open(zip_path).map_err(|e| format!("打开 zip 文件失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("解析 zip 文件失败: {}", e))?;

    archive
        .extract(&extract_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    let new_app = find_app_in_dir(&extract_dir)?;

    remove_quarantine(&new_app)?;

    let current_exe =
        std::env::current_exe().map_err(|e| format!("获取当前程序路径失败: {}", e))?;
    let current_app = current_exe
        .parent()
        .ok_or("无法获取当前程序父目录")?
        .parent()
        .ok_or("无法获取 Contents 目录")?
        .parent()
        .ok_or("无法获取 .app 目录")?
        .to_path_buf();
    let app_parent = current_app
        .parent()
        .ok_or("无法获取 .app 父目录")?
        .to_path_buf();
    let app_name = current_app.file_name().ok_or("无法获取 .app 名称")?;

    let old_app = app_parent.join(format!("{}.old", app_name.to_str().unwrap_or("app")));

    let _ = std::fs::remove_dir_all(&old_app);

    std::fs::rename(&current_app, &old_app).map_err(|e| format!("重命名旧版本失败: {}", e))?;

    let target_app = app_parent.join(app_name);
    std::fs::rename(&new_app, &target_app).map_err(|e| format!("安装新版本失败: {}", e))?;

    let _ = std::fs::remove_dir_all(&extract_dir);
    let _ = std::fs::remove_file(zip_path);

    Ok(target_app)
}

#[cfg(target_os = "macos")]
fn find_app_in_dir(dir: &Path) -> Result<PathBuf, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "app") {
            return Ok(path);
        }
        if path.is_dir()
            && let Ok(found) = find_app_in_dir(&path)
        {
            return Ok(found);
        }
    }
    Err("未找到 .app bundle".to_string())
}

#[cfg(target_os = "macos")]
fn remove_quarantine(path: &Path) -> Result<(), String> {
    let check = std::process::Command::new("xattr")
        .args(["-p", "com.apple.quarantine"])
        .arg(path)
        .output();
    match check {
        Ok(output) if !output.status.success() => return Ok(()),
        Err(_) => return Ok(()),
        _ => {}
    }

    let status = std::process::Command::new("xattr")
        .args(["-d", "-r", "com.apple.quarantine"])
        .arg(path)
        .status()
        .map_err(|e| format!("执行 xattr 失败: {}", e))?;
    if !status.success() {
        return Err(format!(
            "清除 quarantine 属性失败: exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_update_windows(zip_path: &Path) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir();
    let extract_dir = tmp_dir.join(format!("xiaohe-update-{}", std::process::id()));
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {}", e))?;

    let zip_file =
        std::fs::File::open(zip_path).map_err(|e| format!("打开 zip 文件失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("解析 zip 文件失败: {}", e))?;

    archive
        .extract(&extract_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    let new_exe = find_exe_in_dir(&extract_dir)?;

    let current_exe =
        std::env::current_exe().map_err(|e| format!("获取当前程序路径失败: {}", e))?;

    let old_exe = current_exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old_exe);

    let updater_exe = current_exe
        .parent()
        .ok_or("无法获取程序目录")?
        .join("updater.exe");

    if updater_exe.exists() {
        let _ = std::process::Command::new(&updater_exe)
            .arg("--old")
            .arg(&current_exe)
            .arg("--new")
            .arg(&new_exe)
            .spawn()
            .map_err(|e| format!("启动 updater 失败: {}", e))?;
    } else {
        let bat_path = current_exe
            .parent()
            .unwrap()
            .join(format!("xiaohe-restart-{}.bat", std::process::id()));
        let exe_name = current_exe
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let old_str = current_exe.to_string_lossy().to_string();
        let new_str = new_exe.to_string_lossy().to_string();

        let bat_content = format!(
            "@echo off\r\n\
             setlocal enabledelayedexpansion\r\n\
             :wait\r\n\
             tasklist /FI \"IMAGENAME eq {exe}\" /NH 2>nul | find /I \"{exe}\" >nul\r\n\
             if not errorlevel 1 (\r\n\
                 timeout /t 1 /nobreak >nul\r\n\
                 goto wait\r\n\
             )\r\n\
             move /Y \"{old}\" \"{old}.bak\" >nul 2>&1\r\n\
             copy /Y \"{new}\" \"{old}\" >nul 2>&1\r\n\
             start \"\" \"{old}\"\r\n\
             del \"%~f0\"\r\n",
            exe = exe_name,
            old = old_str,
            new = new_str
        );

        std::fs::write(&bat_path, &bat_content).map_err(|e| format!("创建重启脚本失败: {}", e))?;

        std::process::Command::new(&bat_path)
            .spawn()
            .map_err(|e| format!("启动重启脚本失败: {}", e))?;
    }

    let _ = std::fs::remove_dir_all(&extract_dir);
    let _ = std::fs::remove_file(zip_path);

    Ok(current_exe)
}

#[cfg(target_os = "windows")]
fn find_exe_in_dir(dir: &Path) -> Result<PathBuf, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "exe")
            && !path.file_stem().is_some_and(|s| s == "updater")
        {
            return Ok(path);
        }
        if path.is_dir() {
            if let Ok(found) = find_exe_in_dir(&path) {
                return Ok(found);
            }
        }
    }
    Err("未找到 .exe 文件".to_string())
}

pub fn cleanup_old_files_keep_previous() {
    if let Ok(current_exe) = std::env::current_exe() {
        #[cfg(target_os = "macos")]
        {
            if let Some(macos_dir) = current_exe.parent()
                && let Some(contents_dir) = macos_dir.parent()
                && let Some(app_bundle) = contents_dir.parent()
                && let Some(app_parent) = app_bundle.parent()
            {
                let app_name = app_bundle.file_name().unwrap_or_default().to_string_lossy();
                let old_app = app_parent.join(format!("{}.old", app_name));
                let older_app = app_parent.join(format!("{}.old.old", app_name));
                let _ = std::fs::remove_dir_all(&older_app);
                let _ = std::fs::rename(&old_app, &older_app);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let old_exe = current_exe.with_extension("exe.old");
            let older_exe = current_exe.with_extension("exe.old.old");
            let _ = std::fs::remove_file(&older_exe);
            let _ = std::fs::rename(&old_exe, &older_exe);
        }
    }
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
