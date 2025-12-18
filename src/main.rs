use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use regex::Regex;
use ax_server::ax_server;
use axum::{
    routing::{get, post},
    extract::Path,
    response::Html,
    Json, Router,
};
use std::net::SocketAddr;

mod ax_server {
    use axum::Router;
    pub async fn ax_server(listener: tokio::net::TcpListener, app: Router) {
        axum::serve(listener, app).await.unwrap();
    }
}

#[derive(Parser)]
#[command(name = "portkiller")]
#[command(about = "A premium Linux port of Port Killer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all listening ports
    List {
        #[arg(short, long)]
        json: bool,
    },
    /// Output for Waybar module
    Waybar,
    /// Kill a process by PID
    Kill {
        #[arg(value_name = "PID")]
        pid: i32,
    },
    /// Kill all user-owned processes on ports
    KillAll,
    /// Show interactive menu (requires rofi)
    Menu,
    /// Open the full GUI application
    Gui,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PortInfo {
    port: String,
    pid: Option<i32>,
    process_name: Option<String>,
    user: String,
}

fn get_ports() -> Vec<PortInfo> {
    let output = Command::new("ss")
        .args(["-nltp"])
        .output()
        .expect("failed to execute ss");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    let re_users = Regex::new(r#"users:\(\("([^"]+)",pid=(\d+),"#).unwrap();
    let current_user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }

        let local_addr = parts[3];
        let port = local_addr.split(':').last().unwrap_or("").to_string();
        if port.is_empty() { continue; }

        let mut process_name = None;
        let mut pid = None;

        if let Some(caps) = re_users.captures(line) {
            let mut name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            pid = caps.get(2).map(|m| m.as_str().parse::<i32>().ok()).flatten();

            if name.contains('/') {
                name = name.split('/').last().unwrap_or(&name).to_string();
            }
            process_name = Some(name);
        }

        ports.push(PortInfo {
            port,
            pid,
            process_name,
            user: current_user.clone(),
        });
    }

    // Deduplicate and Sort
    ports.retain(|p| p.pid.is_some());
    ports.sort_by(|a, b| a.port.parse::<u32>().unwrap_or(0).cmp(&b.port.parse::<u32>().unwrap_or(0)));
    ports.dedup_by(|a, b| a.port == b.port);
    
    ports
}

async fn run_gui() {
    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("gui.html")) }))
        .route("/api/ports", get(|| async { Json(get_ports()) }))
        .route("/api/kill/{pid}", post(|Path(pid): Path<i32>| async move {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            Json(serde_json::json!({"status": "ok"}))
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9988));
    println!("Launching PortKiller GUI at http://{}", addr);
    
    // Open the browser
    let _ = opener::open(format!("http://{}", addr));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    ax_server(listener, app).await;
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List { json }) => {
            let ports = get_ports();
            if *json {
                println!("{}", serde_json::to_string_pretty(&ports).unwrap());
            } else {
                for p in ports {
                    println!("Port: {}, PID: {:?}, Process: {:?}", p.port, p.pid, p.process_name);
                }
            }
        }
        Some(Commands::Waybar) => {
            let ports = get_ports();
            if ports.is_empty() {
                 println!("{}", serde_json::json!({ "text": "", "tooltip": "No active ports", "class": "empty" }));
                return;
            }

            let text = format!("󰠵 {}", ports.len());
            let mut tooltip = String::from("<b>Active Ports</b>\n");
            for p in &ports {
                tooltip.push_str(&format!("<span color='#a6e3a1'></span>  <b>{}</b>: {} <span color='#6c7086'>(PID: {:?})</span>\n", 
                    p.port, p.process_name.as_deref().unwrap_or("unknown"), p.pid.unwrap_or(0)));
            }

            println!("{}", serde_json::json!({
                "text": text,
                "tooltip": tooltip.trim_end(),
                "class": "active"
            }));
        }
        Some(Commands::Kill { pid }) => {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
        }
        Some(Commands::KillAll) => {
             for p in get_ports() {
                 if let Some(pid) = p.pid {
                     let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
                 }
             }
        }
        Some(Commands::Gui) => {
            run_gui().await;
        }
        Some(Commands::Menu) => {
            let ports = get_ports();
            
            let mut input = String::new();
            input.push_str("󰄬  Open PortKiller GUI                             ⌘O\n");
            input.push_str("----------------------------------------------------\n");
            
            for p in &ports {
                let name = p.process_name.as_deref().unwrap_or("unknown");
                input.push_str(&format!("  <span color='#a6e3a1'></span>  <b>:{}</b>                    {:<15}  <span color='#6c7086'>PID {}</span>\n", 
                    p.port, name, p.pid.unwrap_or(0)));
            }
            
            input.push_str("----------------------------------------------------\n");
            input.push_str("󰑐  Refresh                                       ⌘R\n");
            input.push_str("󰦢  <span color='#f38ba8'>Kill All</span>                                       ⌘K\n");
            input.push_str("󰒓  Settings...                                    ⌘,\n");
            input.push_str("󰈆  Quit PortKiller                                ⌘Q\n");

            let rofi_theme = r#"
                * {
                    bg: #1e1e2e;
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
                    background-color: #11111b;
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
                    placeholder: " Search ports, processes...";
                    placeholder-color: #585b70;
                }
                listview {
                    lines: 12;
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

                if selected.contains("Open PortKiller") {
                    let _ = Command::new("portkiller").arg("gui").spawn();
                } else if selected.contains("Kill All") {
                    let _ = Command::new("pkexec").args(["portkiller", "kill-all"]).status();
                } else if selected.contains("PID") {
                    let re_pid = Regex::new(r"PID (\d+)").unwrap();
                    if let Some(caps) = re_pid.captures(&selected) {
                        let pid = caps.get(1).unwrap().as_str();
                        let _ = Command::new("pkexec").args(["kill", "-9", pid]).status();
                    }
                }
            }
        }
        None => {
            let ports = get_ports();
            for p in ports {
                println!("Port: {}, PID: {:?}, Process: {:?}", p.port, p.pid, p.process_name);
            }
        }
    }
}
