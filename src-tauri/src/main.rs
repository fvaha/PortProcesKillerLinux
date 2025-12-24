// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use regex::Regex;

#[derive(Parser)]
#[command(name = "ppkiller")]
#[command(about = "PP Killer - Port and Process Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the Rofi menu (Slika 1)
    Menu,
    /// Output for Waybar module
    Waybar,
    /// Output all ports in JSON
    List {
        #[arg(short, long)]
        json: bool,
    },
    /// Kill all ports
    KillAll,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        let cli = Cli::try_parse();
        
        if let Ok(cli) = cli {
            match cli.command {
                Some(Commands::Waybar) => {
                    let ports = app_lib::get_ports_list();
                    let processes = app_lib::get_processes_list();
                    
                    // Get top 10 processes by CPU or memory
                    let mut top_processes: Vec<_> = processes.iter()
                        .filter_map(|p| {
                            let cpu = p.cpu.parse::<f64>().ok()?;
                            let mem = p.mem.parse::<f64>().ok()?;
                            Some((p, cpu, mem))
                        })
                        .collect();
                    top_processes.sort_by(|a, b| {
                        (b.1 + b.2 / 10.0).partial_cmp(&(a.1 + a.2 / 10.0)).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let top_10: Vec<_> = top_processes.iter().take(10).map(|(p, _, _)| p).collect();
                    
                    let port_count = ports.len();
                    let process_count = processes.len();
                    
                    if port_count == 0 && process_count == 0 {
                        println!("{}", serde_json::json!({ "text": "", "tooltip": "No active ports or processes", "class": "empty" }));
                        return;
                    }
                    
                    let text = format!("󰠵 {} | 󰍛 {}", port_count, process_count);
                    let mut tooltip = String::from("<b>PP Killer</b>\n");
                    tooltip.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    tooltip.push_str(&format!("<b>Active Ports: {}</b>\n", port_count));
                    for p in &ports {
                        tooltip.push_str(&format!("<span color='#a6e3a1'></span>  <b>:{}</b> {} <span color='#6c7086'>(PID: {})</span>\n", 
                            p.port, p.process_name.as_deref().unwrap_or("unknown"), p.pid.unwrap_or(0)));
                    }
                    tooltip.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    tooltip.push_str(&format!("<b>Top Processes (by CPU/Memory):</b>\n"));
                    for p in top_10 {
                        let cpu = p.cpu.parse::<f64>().unwrap_or(0.0);
                        let mem = p.mem.parse::<f64>().unwrap_or(0.0);
                        let mem_display = if mem >= 1024.0 {
                            format!("{:.1}GB", mem / 1024.0)
                        } else {
                            format!("{:.1}MB", mem)
                        };
                        tooltip.push_str(&format!("<span color='#f9e2af'>󰍛</span>  <b>{}</b> CPU: {:.1}% Mem: {} <span color='#6c7086'>(PID: {})</span>\n", 
                            p.name, cpu, mem_display, p.pid));
                    }
                    
                    println!("{}", serde_json::json!({
                        "text": text, "tooltip": tooltip.trim_end(), "class": "active"
                    }));
                    return;
                }
                Some(Commands::List { json }) => {
                    let ports = app_lib::get_ports_list();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&ports).unwrap());
                    } else {
                        for p in ports {
                            println!("Port: {}, PID: {:?}, Process: {:?}", p.port, p.pid, p.process_name);
                        }
                    }
                    return;
                }
                Some(Commands::KillAll) => {
                    app_lib::kill_all_ports();
                    return;
                }
                Some(Commands::Menu) => {
                    run_menu();
                    return;
                }
                None => {}
            }
        }
    }

    // Default: launch GUI
    app_lib::run();
}

fn run_menu() {
    let ports = app_lib::get_ports_list();
    let processes = app_lib::get_processes_list();
    
    // Get top 10 processes by CPU/memory
    let mut top_processes: Vec<_> = processes.iter()
        .filter_map(|p| {
            let cpu = p.cpu.parse::<f64>().ok()?;
            let mem = p.mem.parse::<f64>().ok()?;
            Some((p, cpu, mem))
        })
        .collect();
    top_processes.sort_by(|a, b| {
        (b.1 + b.2 / 10.0).partial_cmp(&(a.1 + a.2 / 10.0)).unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_10: Vec<_> = top_processes.iter().take(10).map(|(p, _, _)| p).collect();
    
    let mut input = String::new();
    input.push_str("󰄬  Open PP Killer GUI                             ⌘O\n");
    input.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    input.push_str("<b>󰠵 PORTS</b>                                    <span color='#6c7086'>Tab 1</span>\n");
    input.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    if ports.is_empty() {
        input.push_str("  <span color='#6c7086'>No active ports</span>\n");
    } else {
        for p in &ports {
            let name = p.process_name.as_deref().unwrap_or("unknown");
            input.push_str(&format!("  <span color='#a6e3a1'></span>  <b>:{}</b>                    {:<15}  <span color='#6c7086'>PID {}</span>\n", 
                p.port, name, p.pid.unwrap_or(0)));
        }
    }
    
    input.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    input.push_str("<b>󰍛 PROCESSES</b>                               <span color='#6c7086'>Tab 2</span>\n");
    input.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    if top_10.is_empty() {
        input.push_str("  <span color='#6c7086'>No processes</span>\n");
    } else {
        for p in top_10 {
            let cpu = p.cpu.parse::<f64>().unwrap_or(0.0);
            let mem = p.mem.parse::<f64>().unwrap_or(0.0);
            let mem_display = if mem >= 1024.0 {
                format!("{:.1}GB", mem / 1024.0)
            } else {
                format!("{:.1}MB", mem)
            };
            input.push_str(&format!("  <span color='#f9e2af'>󰍛</span>  <b>{}</b>  CPU: {:.1}%  Mem: {}  <span color='#6c7086'>PID {}</span>\n", 
                p.name, cpu, mem_display, p.pid));
        }
    }
    
    input.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    input.push_str("󰑐  Refresh                                       ⌘R\n");
    input.push_str("󰦢  <span color='#f38ba8'>Kill All Ports</span>                              ⌘K\n");
    input.push_str("󰈆  Quit                                          ⌘Q\n");

    let rofi_theme = r#"
        * {
            bg: #11111b;
            fg: #cdd6f4;
            accent: #f38ba8;
            font: "JetBrainsMono Nerd Font 10";
        }
        window {
            width: 480px;
            border: 1px;
            border-radius: 12px;
            border-color: #313244;
            background-color: @bg;
            padding: 0px;
        }
        mainbox {
            children: [ inputbar, listview ];
            padding: 10px;
        }
        inputbar {
            background-color: #1e1e2e;
            border-radius: 8px;
            padding: 8px 12px;
            margin: 0 0 10px 0;
            children: [ prompt, entry ];
        }
        prompt {
            content: "󰩠";
            text-color: #f5c2e7;
        }
        entry {
            placeholder: " Search port or process...";
            placeholder-color: #585b70;
        }
        listview {
            lines: 10;
            scrollbar: false;
        }
        element {
            padding: 8px 12px;
            border-radius: 6px;
        }
        element selected {
            background-color: #313244;
            text-color: #89b4fa;
        }
    "#;

    let rofi = Command::new("rofi")
        .args(["-dmenu", "-p", "", "-i", "-markup-rows", "-theme-str", rofi_theme])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(mut child) = rofi {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input.as_bytes());
        }

        let output = child.wait_with_output().expect("failed read rofi");
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Find AppImage path
        let appimage_path = std::env::var("APPIMAGE")
            .or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                let possible_paths = vec![
                    format!("{}/PP-Killer-x86_64.AppImage", std::env::current_dir().unwrap_or_default().display()),
                    format!("{}/PortKiller-x86_64.AppImage", std::env::current_dir().unwrap_or_default().display()),
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
            .unwrap_or_else(|_| "ppkiller".to_string());
        
        if selected.contains("Open PP Killer") {
            // Try to launch AppImage, fallback to command
            if appimage_path.ends_with(".AppImage") {
                let _ = Command::new(&appimage_path).spawn();
            } else {
                let _ = Command::new(&appimage_path).spawn();
            }
        } else if selected.contains("Kill All") {
            let _ = Command::new("pkexec").args([&appimage_path, "kill-all"]).status();
        } else if selected.contains("PID") {
            let re = Regex::new(r"PID (\d+)").unwrap();
            if let Some(caps) = re.captures(&selected) {
                let pid = caps.get(1).unwrap().as_str();
                let _ = Command::new("pkexec").args(["kill", "-9", pid]).status();
            }
        }
    }
}
