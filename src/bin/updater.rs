#[cfg(target_os = "windows")]
use std::{env, fs, path::PathBuf, process::Command, thread, time::Duration};

#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut old_path: Option<PathBuf> = None;
    let mut new_path: Option<PathBuf> = None;

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

    if !wait_for_process_exit(&old_path) {
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

    let _ = Command::new(&old_path).spawn();
}

#[cfg(target_os = "windows")]
fn wait_for_process_exit(exe_path: &PathBuf) -> bool {
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

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("updater is only supported on Windows");
}
