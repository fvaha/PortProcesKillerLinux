use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use sysinfo::{Pid, System, ProcessesToUpdate, Users};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortInfo {
    pub port: String,
    pub pid: Option<i32>,
    pub process_name: Option<String>,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu: String,
    pub mem: String,
    pub user: String,
}

// Helper to read /proc/net/tcp and tcp6
fn scan_proc_net_tcp(file: &str) -> Vec<(u16, i32)> {
    let mut results = Vec::new();
    if let Ok(content) = fs::read_to_string(file) {
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 { continue; }
            
            // State 0A means LISTEN
            if parts[3] != "0A" { continue; }
            
            // Parse port from hex "00000000:1F90" -> 1F90 is port
            let local_addr = parts[1];
            if let Some(pos) = local_addr.find(':') {
                if let Ok(port) = u16::from_str_radix(&local_addr[pos+1..], 16) {
                    // Inode is at index 9
                    if let Ok(inode) = parts[9].parse::<i32>() {
                         results.push((port, inode));
                    }
                }
            }
        }
    }
    results
}

// Map inode to PID by scanning /proc/[pid]/fd
fn get_pids_for_inodes(inodes: &HashSet<i32>) -> HashMap<i32, i32> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(pid) = file_name.parse::<i32>() {
                    let fd_path = format!("/proc/{}/fd", pid);
                    if let Ok(fd_entries) = fs::read_dir(fd_path) {
                        for fd in fd_entries.flatten() {
                            if let Ok(target) = fs::read_link(fd.path()) {
                                if let Some(target_str) = target.to_str() {
                                    if target_str.starts_with("socket:[") {
                                        let inode_str = &target_str[8..target_str.len()-1];
                                        if let Ok(inode) = inode_str.parse::<i32>() {
                                            if inodes.contains(&inode) {
                                                map.insert(inode, pid);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

// --- Implementation Functions (Not Tauri Commands) ---

fn get_ports_impl() -> Vec<PortInfo> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let users = Users::new_with_refreshed_list();

    let mut ports_map: HashMap<String, PortInfo> = HashMap::new();
    
    // 1. Get all listening ports and their inodes
    let mut tcp4 = scan_proc_net_tcp("/proc/net/tcp");
    let tcp6 = scan_proc_net_tcp("/proc/net/tcp6");
    tcp4.extend(tcp6);
    
    let inodes: HashSet<i32> = tcp4.iter().map(|(_, inode)| *inode).collect();
    
    // 2. Find PIDs for these inodes
    let inode_pid_map = get_pids_for_inodes(&inodes);
    
    for (port, inode) in tcp4 {
        let port_str = port.to_string();
        let pid = inode_pid_map.get(&inode).copied();
        
        let mut process_name = None;
        let mut user = "unknown".to_string();

        if let Some(p) = pid {
            let pid_u32 = p as usize;
            if let Some(process) = system.process(Pid::from(pid_u32)) {
                // Fix: Process::name returns &OsStr, needs to_string_lossy
                process_name = Some(process.name().to_string_lossy().into_owned());
                
                if let Some(uid) = process.user_id() {
                    if let Some(u) = users.get_user_by_id(uid) {
                        // Fix: User::name returns &str, needs to_string
                        user = u.name().to_string();
                    }
                }
            } else {
                user = "root/system".to_string(); 
            }
        } else {
            user = if port < 1024 { "root".to_string() } else { "unknown".to_string() };
        }

        ports_map.insert(port_str.clone(), PortInfo {
            port: port_str,
            pid,
            process_name,
            user,
        });
    }

    let mut ports: Vec<PortInfo> = ports_map.into_values().collect();
    ports.sort_by_key(|p| p.port.parse::<u32>().unwrap_or(0));
    ports
}

fn kill_port_impl(pid: i32) -> bool {
    let s = System::new(); 
    if let Some(process) = s.process(Pid::from(pid as usize)) {
        return process.kill();
    }
    
    // Fallback
    let _ = Command::new("kill").arg(pid.to_string()).status();
    let status = Command::new("kill").arg("-9").arg(pid.to_string()).status();
    match status {
         Ok(s) => s.success(),
         Err(_) => false,
    }
}

// --- Tauri Commands ---

#[tauri::command]
fn get_ports() -> Vec<PortInfo> {
    get_ports_impl()
}

#[tauri::command]
fn get_processes() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let users = Users::new_with_refreshed_list();
    
    let mut processes = Vec::new();
    
    for (pid, process) in system.processes() {
        let mut user = "unknown".to_string();
        if let Some(uid) = process.user_id() {
             if let Some(u) = users.get_user_by_id(uid) {
                 user = u.name().to_string();
             }
        }

        processes.push(ProcessInfo {
            pid: pid.as_u32() as i32,
            name: process.name().to_string_lossy().into_owned(),
            cpu: format!("{:.1}", process.cpu_usage()),
            mem: format!("{:.1}", (process.memory() as f64 / 1024.0 / 1024.0)), // MB
            user,
        });
    }
    
    processes.sort_by(|a, b| {
        let cpu_a = a.cpu.parse::<f64>().unwrap_or(0.0);
        let cpu_b = b.cpu.parse::<f64>().unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    processes.truncate(100);
    processes
}

#[tauri::command]
fn kill_port(pid: i32) -> bool {
    kill_port_impl(pid)
}

#[tauri::command]
fn kill_process(pid: i32) -> bool {
    kill_port_impl(pid)
}

#[tauri::command]
fn open_terminal() {
    let terms = ["gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "foot", "tilix", "termite", "xterm"];
    for term in terms {
        if Command::new(term).spawn().is_ok() {
            break;
        }
    }
}

#[tauri::command]
fn setup_waybar() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Could not find HOME dir")?;
    let config_dir = PathBuf::from(&home).join(".config/waybar");
    
    if !config_dir.exists() {
        return Err("Waybar config directory not found at ~/.config/waybar".to_string());
    }

    let config_path = config_dir.join("config");
    let mut config = fs::read_to_string(&config_path).map_err(|_| "Could not read waybar config")?;
    
    // Find AppImage path - check common locations
    let appimage_path = std::env::var("APPIMAGE")
        .or_else(|_| {
            // Try to find AppImage in common locations
            let possible_paths = vec![
                format!("{}/PortKiller-x86_64.AppImage", std::env::current_dir().unwrap_or_default().display()),
                format!("{}/PP-Killer-x86_64.AppImage", std::env::current_dir().unwrap_or_default().display()),
                "/opt/ppkiller/PP-Killer-x86_64.AppImage".to_string(),
                format!("{}/.local/bin/PP-Killer-x86_64.AppImage", home),
            ];
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| format!("{}/.local/bin/ppkiller", home)); // Fallback to local bin
    
    let mut config_changed = false;
    
    // Dodaj definiciju modula ako ne postoji
    if !config.contains("\"custom/ppkiller\"") {
        let module_def = format!(r#"
    "custom/ppkiller": {{
        "format": "{{}}",
        "exec": "{} waybar",
        "return-type": "json",
        "on-click": "{} menu",
        "on-click-right": "{}",
        "interval": 5,
        "tooltip": true
    }},"#, appimage_path, appimage_path, appimage_path);

        if let Some(pos) = config.find('}') {
            config.insert_str(pos + 1, &module_def);
            config_changed = true;
        } else {
            return Err("Waybar config format is invalid".to_string());
        }
    }
    
    // Proveri da li je modul već dodat u modules-right ili modules-left
    let module_in_list = {
        // Proveri da li postoji u modules-right ili modules-left listi
        let modules_right_start = config.find("\"modules-right\":");
        let modules_left_start = config.find("\"modules-left\":");
        
        let check_in_section = |start_pos: Option<usize>| -> bool {
            if let Some(start) = start_pos {
                if let Some(bracket_start) = config[start..].find('[') {
                    let list_start = start + bracket_start;
                    if let Some(bracket_end) = config[list_start..].find(']') {
                        let list_end = list_start + bracket_end;
                        let section = &config[list_start..list_end];
                        return section.contains("\"custom/ppkiller\"");
                    }
                }
            }
            false
        };
        
        check_in_section(modules_right_start) || check_in_section(modules_left_start)
    };
    
    if !module_in_list {
        // Uvek dodaj u modules-right
        if let Some(pos) = config.find("\"modules-right\":") {
            if let Some(bracket_pos) = config[pos..].find('[') {
                let insert_pos = pos + bracket_pos + 1;
                // Proveri da li već postoji neki modul u listi
                if let Some(first_module) = config[insert_pos..].find('"') {
                    let actual_pos = insert_pos + first_module;
                    config.insert_str(actual_pos, "\"custom/ppkiller\",\n        ");
                    config_changed = true;
                } else {
                    // Prazna lista
                    config.insert_str(insert_pos + 1, "\"custom/ppkiller\"");
                    config_changed = true;
                }
            }
        } else {
            // Ako nema modules-right sekcije, kreiraj je
            // Pronađi gde da je dodamo (najbolje posle modules-center ili na kraju)
            if let Some(pos) = config.find("\"modules-center\":") {
                if let Some(closing_bracket) = config[pos..].find(']') {
                    let insert_pos = pos + closing_bracket + 1;
                    config.insert_str(insert_pos, ",\n\n  \"modules-right\": [\n    \"custom/ppkiller\"\n  ],");
                    config_changed = true;
                }
            } else {
                // Dodaj na kraju pre zadnje zagrade
                if let Some(pos) = config.rfind('}') {
                    config.insert_str(pos, ",\n\n  \"modules-right\": [\n    \"custom/ppkiller\"\n  ]");
                    config_changed = true;
                }
            }
        }
    }
    
    if config_changed {
        fs::write(&config_path, config).map_err(|_| "Failed to write waybar config")?;
    }

    let css_path = config_dir.join("style.css");
    let mut css = fs::read_to_string(&css_path).unwrap_or_default();
    if !css.contains("#custom-ppkiller") {
        let css_append = r#"
#custom-ppkiller {
    background: rgba(59, 130, 246, 0.1);
    color: #60a5fa;
    border-radius: 8px;
    padding: 0 10px;
    margin: 4px 2px;
}
#custom-ppkiller.active {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}
"#;
        css.push_str(css_append);
        fs::write(&css_path, css).map_err(|_| "Failed to write style.css")?;
    }

    let _ = Command::new("killall").arg("-SIGUSR2").arg("waybar").status();

    if module_in_list {
        Ok("Waybar integrated successfully! The 'custom/ppkiller' module has been added to your Waybar configuration.".to_string())
    } else {
        Ok("Waybar integrated! The 'custom/ppkiller' module definition has been added. Please manually add 'custom/ppkiller' to your 'modules-right' or 'modules-left' in waybar config if it's not showing.".to_string())
    }
}

// --- Public helpers for main.rs ---

pub fn get_ports_list() -> Vec<PortInfo> {
    get_ports_impl()
}

pub fn get_processes_list() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let users = Users::new_with_refreshed_list();
    
    let mut processes = Vec::new();
    
    for (pid, process) in system.processes() {
        let mut user = "unknown".to_string();
        if let Some(uid) = process.user_id() {
             if let Some(u) = users.get_user_by_id(uid) {
                 user = u.name().to_string();
             }
        }

        processes.push(ProcessInfo {
            pid: pid.as_u32() as i32,
            name: process.name().to_string_lossy().into_owned(),
            cpu: format!("{:.1}", process.cpu_usage()),
            mem: format!("{:.1}", (process.memory() as f64 / 1024.0 / 1024.0)), // MB
            user,
        });
    }
    
    processes.sort_by(|a, b| {
        let cpu_a = a.cpu.parse::<f64>().unwrap_or(0.0);
        let cpu_b = b.cpu.parse::<f64>().unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    processes.truncate(100);
    processes
}

pub fn kill_all_ports() {
    let ports = get_ports_impl();
    for p in ports {
        if let Some(pid) = p.pid {
            let _ = kill_port_impl(pid);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_ports, get_processes, kill_port, kill_process, open_terminal, setup_waybar])
        .setup(|_app| {
            // Open devtools in development mode
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(window) = _app.get_webview_window("main") {
                    let _ = window.open_devtools();
                    println!("DevTools opened automatically (debug mode)");
                } else {
                    println!("Warning: Could not find 'main' window to open DevTools");
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
