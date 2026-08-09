#[cfg(target_os = "windows")]
use std::{env, fs, path::PathBuf, process::Command, thread, time::Duration};

#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut old_path: Option<PathBuf> = None;
    let mut new_path: Option<PathBuf> = None;
    let mut pid: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--old" => {
                i += 1;
                if i < args.len() {
                    old_path = Some(PathBuf::from(&args[i]));
                }
            }
            "--new" => {
                i += 1;
                if i < args.len() {
                    new_path = Some(PathBuf::from(&args[i]));
                }
            }
            "--pid" => {
                i += 1;
                if i < args.len() {
                    pid = args[i].parse::<u32>().ok();
                }
            }
            _ => {}
        }
        i += 1;
    }

    let old_path = match old_path {
        Some(p) => p,
        None => {
            eprintln!("updater: missing --old argument");
            return;
        }
    };

    let new_path = match new_path {
        Some(p) => p,
        None => {
            eprintln!("updater: missing --new argument");
            return;
        }
    };

    if !wait_for_process_exit(&old_path, pid) {
        eprintln!("updater: timeout waiting for process to exit after 60s");
    }

    let old_bak = old_path.with_extension("exe.old");
    let _ = fs::remove_file(&old_bak);

    for attempt in 0..5 {
        match fs::rename(&old_path, &old_bak) {
            Ok(_) => break,
            Err(e) => {
                if attempt < 4 {
                    thread::sleep(Duration::from_secs(1));
                } else {
                    eprintln!("updater: failed to rename old exe after 5 attempts: {}", e);
                    return;
                }
            }
        }
    }

    for attempt in 0..5 {
        match fs::copy(&new_path, &old_path) {
            Ok(_) => break,
            Err(e) => {
                if attempt < 4 {
                    thread::sleep(Duration::from_secs(1));
                } else {
                    eprintln!("updater: failed to copy new exe after 5 attempts: {}", e);
                    return;
                }
            }
        }
    }

    // 只有用户选择了「立即重启」（写入了 relaunch 标记）才启动新版本。
    // 若标记不存在（用户选了「稍后」或进程被直接关闭），仅完成文件替换、不自动
    // 重启，避免「关闭 app 后旧版本被自动拉起新版本」的问题。
    let marker = old_path.with_extension("exe.relaunch");
    if marker.exists() {
        let _ = fs::remove_file(&marker);
        let _ = Command::new(&old_path).spawn();
    }
}

#[cfg(target_os = "windows")]
fn wait_for_process_exit(exe_path: &PathBuf, pid: Option<u32>) -> bool {
    // 优先按 PID 精确等待当前 app 进程退出；若拿不到 PID，则回退到按进程名等待。
    // 按 PID 可避免用户同时开多个同名实例时，按进程名等待被其它实例拖死
    // （导致替换迟迟不执行）。
    if let Some(pid) = pid {
        if !is_process_running(pid) {
            return true;
        }
        for _ in 0..120 {
            thread::sleep(Duration::from_millis(500));
            if !is_process_running(pid) {
                return true;
            }
        }
        return false;
    }

    let exe_name = exe_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for _ in 0..120 {
        thread::sleep(Duration::from_millis(500));

        let output = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", exe_name), "/NH"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let running: Vec<&str> = stdout.lines().filter(|l| l.contains(&exe_name)).collect();

                if running.is_empty() {
                    return true;
                }
            }
            Err(_) => {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_process_running(pid: u32) -> bool {
    // 用 tasklist 按 PID 精确查询该进程是否还存在；失败/无结果视为已退出。
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().any(|l| l.contains(&pid.to_string()))
        }
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("updater is only supported on Windows");
}
