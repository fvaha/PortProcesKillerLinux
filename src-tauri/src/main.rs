// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "portkiller")]
#[command(about = "A premium Linux port of Port Killer", long_about = None)]
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
            placeholder: " Search ports, processes...";
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

        if selected.contains("Open PortKiller") {
            let _ = Command::new("portkiller").spawn(); // Open GUI
        } else if selected.contains("Kill All") {
            let _ = Command::new("pkexec").args(["portkiller", "kill-all"]).status();
        } else if selected.contains("PID") {
            let re = regex::Regex::new(r"PID (\d+)").unwrap();
            if let Some(caps) = re.captures(&selected) {
                let pid = caps.get(1).unwrap().as_str();
                let _ = Command::new("pkexec").args(["kill", "-9", pid]).status();
            }
        }
    }
}
