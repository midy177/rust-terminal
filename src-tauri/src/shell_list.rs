use std::env;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemShell {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub cwd: PathBuf,
    // pub icon: String,
}

fn start_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn get_short_hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| String::new())
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use super::*;
    use nix::unistd::{fork, ForkResult};
    use std::fs::File;
    use std::io::{BufRead, BufReader, Lines};
    use std::process::Command;

    pub fn support_login() -> bool {
        let user = env::var("USER").unwrap_or_default();
        if user.is_empty() {
            return false;
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child: _ }) => {
                // Parent process
                true
            }
            Ok(ForkResult::Child) => {
                // Child process
                let args = vec!["-f", &user];
                Command::new("login")
                    .args(&args)
                    .spawn()
                    .expect("failed to execute login")
                    .wait()
                    .expect("failed to wait on login");
                std::process::exit(0);
            }
            Err(_) => false,
        }
    }

    pub fn get_shells() -> Vec<SystemShell> {
        let mut shells = Vec::new();
        let sbl = support_login();
        let term = env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
        let home = start_dir();
        let user = env::var("USER").unwrap_or_default();
        let hostname = get_short_hostname();
        let title = if hostname.is_empty() {
            user.clone()
        } else {
            format!("{}@{}", user,
                    hostname.clone().split_once('.').map_or(hostname, |(before, _)| before.to_string()))
        };

        if sbl && !user.is_empty() {
            shells.push(SystemShell {
                name: title,
                command: "login".to_string(),
                args: vec!["-f".to_string(), user],
                env: vec![format!("TERM={}", term)],
                cwd: home.clone(),
            });
        }
        // else {
            if let Ok(file) = File::open("/etc/shells") {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    match line {
                        Ok(line) if !line.starts_with('#') && !line.is_empty() => {
                            if let Some(last_part) = line.split('/').last() {
                                shells.push(SystemShell {
                                    name: last_part.to_string(),
                                    command: line.clone(),
                                    args: Vec::new(),
                                    env: vec![format!("TERM={}", term)],
                                    cwd: home.clone(),
                                });
                            }
                        },
                        _ => continue,
                    }
                }
            }
        // }

        shells
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;
    use which::which;

    pub fn get_shells() -> Vec<SystemShell> {
        let mut shells = Vec::new();
        let term = env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
        let home = start_dir();
        let user = env::var("USERNAME").unwrap_or_default();
        let hostname = get_short_hostname();
        let title = if hostname.is_empty() {
            user.clone()
        } else {
            format!("{}@{}", user, hostname)
        };

        // Add CMD
        shells.push(SystemShell {
            name: "CMD".to_string(),
            command: "cmd.exe".to_string(),
            args: Vec::new(),
            env: vec![format!("TERM={}", term)],
            cwd: home.clone(),
            // icon: "/assets/icons/windows.svg".to_string(),
        });

        // Add PowerShell
        if let Ok(powershell_path) = which("powershell.exe") {
            shells.push(SystemShell {
                name: "PowerShell".to_string(),
                command: powershell_path.to_string_lossy().to_string(),
                args: Vec::new(),
                env: vec![format!("TERM={}", term)],
                cwd: home.clone(),
                // icon: "/assets/icons/powershell.svg".to_string(),
            });
        }

        // Check for Git Bash
        let git_bash_path = Path::new("C:\\Program Files\\Git\\bin\\bash.exe");
        if git_bash_path.exists() {
            shells.push(SystemShell {
                name: "Git Bash".to_string(),
                command: git_bash_path.to_string_lossy().to_string(),
                args: Vec::new(),
                env: vec![format!("TERM={}", term)],
                cwd: home.clone(),
                // icon: "/assets/icons/git-bash.svg".to_string(),
            });
        }

        // Check for WSL distributions
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(lxss) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss") {
            for key in lxss.enum_keys().filter_map(|x| x.ok()) {
                if let Ok(subkey) = lxss.open_subkey(&key) {
                    if let Ok(distribution_name) = subkey.get_value::<String, _>("DistributionName") {
                        shells.push(SystemShell {
                            name: format!("WSL({})", distribution_name),
                            command: "wsl.exe".to_string(),
                            args: vec!["-d".to_string(), distribution_name],
                            env: vec![
                                "TERM=xterm-256color".to_string(),
                                "COLORTERM=truecolor".to_string(),
                            ],
                            cwd: home.clone(),
                            // icon: "/assets/icons/linux.svg".to_string(),
                        });
                    }
                }
            }
        }

        shells
    }
}

#[cfg(not(target_os = "windows"))]
pub use unix::get_shells;

#[cfg(target_os = "windows")]
pub use windows::get_shells;
pub fn get_available_shells() -> Vec<SystemShell> {
    get_shells()
}