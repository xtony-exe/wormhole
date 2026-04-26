use std::net::IpAddr;
use std::time::Duration;

use anyhow::Result;
use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use colored::Colorize;
use wormhole::{client::Client, server::Server};

// ─── ASCII Banner ─────────────────────────────────────────────────────────────

const BANNER: &str = r"
 ░██╗░░░░░░░██╗░█████╗░██████╗░███╗░░░███╗██╗░░██╗░█████╗░██╗░░░░░███████╗
 ░██║░░██╗░░██║██╔══██╗██╔══██╗████╗░████║██║░░██║██╔══██╗██║░░░░░██╔════╝
 ░╚██╗████╗██╔╝██║░░██║██████╔╝██╔████╔██║███████║██║░░██║██║░░░░░█████╗░░
 ░░████╔═████║░██║░░██║██╔══██╗██║╚██╔╝██║██╔══██║██║░░██║██║░░░░░██╔══╝░░
 ░░╚██╔╝░╚██╔╝░╚█████╔╝██║░░██║██║░╚═╝░██║██║░░██║╚█████╔╝███████╗███████╗
 ░░░╚═╝░░░╚═╝░░╚════╝░╚═╝░░╚═╝╚═╝░░░░░╚═╝╚═╝░░╚═╝░╚════╝░╚══════╝╚══════╝";

fn print_banner() {
    println!("{}", BANNER.bright_cyan().bold());
    println!(
        "  {}  {}  {}\n",
        "v1.0.0".bright_white().bold(),
        "—".dimmed(),
        "by THINKING TEAM · XTONY".bright_magenta().bold()
    );
}

// ─── CLI Definition ───────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[clap(author = "XTONY", version = "1.0.0", about = "Wormhole — A modern TCP tunnel by THINKING TEAM")]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Open a wormhole — expose a local port to the internet.
    Open {
        /// The local port to expose.
        #[clap(env = "WORMHOLE_LOCAL_PORT")]
        local_port: u16,

        /// The local host to expose.
        #[clap(short, long, value_name = "HOST", default_value = "localhost")]
        local_host: String,

        /// Address of the remote wormhole server.
        #[clap(short, long, env = "WORMHOLE_SERVER")]
        to: String,

        /// Optional port on the remote server to select (0 = auto-assign).
        #[clap(short, long, default_value_t = 0)]
        port: u16,

        /// Optional secret for authentication.
        #[clap(short, long, env = "WORMHOLE_SECRET", hide_env_values = true)]
        secret: Option<String>,

        /// A friendly label for this tunnel (displayed on connect).
        #[clap(long, value_name = "NAME")]
        label: Option<String>,

        /// Auto-reconnect on disconnect with exponential backoff.
        #[clap(short, long)]
        retry: bool,
    },

    /// Host a wormhole server — relay traffic for connected clients.
    Host {
        /// Minimum accepted TCP port number.
        #[clap(long, default_value_t = 1024, env = "WORMHOLE_MIN_PORT")]
        min_port: u16,

        /// Maximum accepted TCP port number.
        #[clap(long, default_value_t = 65535, env = "WORMHOLE_MAX_PORT")]
        max_port: u16,

        /// Optional secret for authentication.
        #[clap(short, long, env = "WORMHOLE_SECRET", hide_env_values = true)]
        secret: Option<String>,

        /// IP address to bind to (clients must reach this).
        #[clap(long, default_value = "0.0.0.0")]
        bind_addr: IpAddr,

        /// IP address where tunnels will listen on, defaults to --bind-addr.
        #[clap(long)]
        bind_tunnels: Option<IpAddr>,

        /// Maximum number of simultaneous client tunnels.
        #[clap(long, default_value_t = 100)]
        max_clients: usize,
    },
}

// ─── Runner ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn run(command: Command) -> Result<()> {
    match command {
        // ── Client Mode ──────────────────────────────────────────────────────
        Command::Open {
            local_host,
            local_port,
            to,
            port,
            secret,
            label,
            retry,
        } => {
            let label_str = label.as_deref().unwrap_or("wormhole");

            println!(
                "  {} Opening {} on {}:{}\n",
                "◈".bright_cyan().bold(),
                format!("[{}]", label_str).bright_yellow().bold(),
                local_host.bright_white(),
                local_port.to_string().bright_white()
            );

            let mut attempt = 0u32;

            loop {
                match Client::new(
                    &local_host,
                    local_port,
                    &to,
                    port,
                    secret.as_deref(),
                )
                .await
                {
                    Ok(client) => {
                        attempt = 0;

                        let remote_port = client.remote_port();

                        // ── Connection Info Box ───────────────────────────
                        println!(
                            "  {}",
                            "┌──────────────────────────────────────────┐"
                                .bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            "✔ WORMHOLE ACTIVE".bright_green().bold(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}",
                            "├──────────────────────────────────────────┤"
                                .bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!("  Label  :  {}", label_str).bright_yellow(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!(
                                "  Server :  {}",
                                to
                            )
                            .bright_white()
                            .bold(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!("  IP     :  {}", to).bright_white().bold(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!("  Port   :  {}", remote_port)
                                .bright_green()
                                .bold(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!("  Local  :  {}:{}", local_host, local_port)
                                .bright_white(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}  {} {}",
                            "│".bright_cyan(),
                            format!("  URL    :  {}:{}", to, remote_port)
                                .bright_cyan()
                                .underline()
                                .bold(),
                            "│".bright_cyan()
                        );
                        println!(
                            "  {}",
                            "└──────────────────────────────────────────┘"
                                .bright_cyan()
                        );
                        println!();

                        // ── QR Code ───────────────────────────────────────
                        let qr_url = format!("tcp://{}:{}", to, remote_port);
                        println!("  {} Scan to connect:\n", "◉".bright_yellow().bold());
                        if let Err(e) = qr2term::print_qr(&qr_url) {
                            println!("  (QR unavailable: {})", e);
                        }
                        println!();

                        println!(
                            "  {} Forwarding traffic... {}\n",
                            "▶".bright_blue().bold(),
                            "(Ctrl+C to stop)".dimmed()
                        );

                        // ── Listen ────────────────────────────────────────
                        match client.listen().await {
                            Ok(_) => {
                                if !retry {
                                    break;
                                }
                                println!(
                                    "\n  {} Connection closed.",
                                    "⚠".bright_yellow().bold()
                                );
                            }
                            Err(err) => {
                                if !retry {
                                    return Err(err);
                                }
                                println!(
                                    "\n  {} Connection lost: {}",
                                    "✗".bright_red().bold(),
                                    err.to_string().dimmed()
                                );
                            }
                        }
                    }
                    Err(err) => {
                        if !retry {
                            return Err(err);
                        }
                        println!(
                            "\n  {} Failed to connect: {}",
                            "✗".bright_red().bold(),
                            err.to_string().dimmed()
                        );
                    }
                }

                // ── Exponential backoff (max 32s) ─────────────────────────
                attempt += 1;
                let wait = Duration::from_secs(2u64.pow(attempt.min(5)));
                println!(
                    "  {} Retrying in {}s (attempt #{})...\n",
                    "↺".bright_cyan(),
                    wait.as_secs(),
                    attempt
                );
                tokio::time::sleep(wait).await;
            }
        }

        // ── Server Mode ──────────────────────────────────────────────────────
        Command::Host {
            min_port,
            max_port,
            secret,
            bind_addr,
            bind_tunnels,
            max_clients,
        } => {
            let port_range = min_port..=max_port;
            if port_range.is_empty() {
                Args::command()
                    .error(ErrorKind::InvalidValue, "port range is empty")
                    .exit();
            }

            println!(
                "  {}",
                "┌──────────────────────────────────────────┐"
                    .bright_cyan()
            );
            println!(
                "  {}  {} {}",
                "│".bright_cyan(),
                "◈ WORMHOLE SERVER".bright_cyan().bold(),
                "│".bright_cyan()
            );
            println!(
                "  {}",
                "├──────────────────────────────────────────┤"
                    .bright_cyan()
            );
            println!(
                "  {}  {} {}",
                "│".bright_cyan(),
                format!("  Bind IP     :  {}", bind_addr).bright_white().bold(),
                "│".bright_cyan()
            );
            println!(
                "  {}  {} {}",
                "│".bright_cyan(),
                format!("  Control Port:  7835").bright_white(),
                "│".bright_cyan()
            );
            println!(
                "  {}  {} {}",
                "│".bright_cyan(),
                format!("  Port Range  :  {} – {}", min_port, max_port)
                    .bright_yellow(),
                "│".bright_cyan()
            );
            println!(
                "  {}  {} {}",
                "│".bright_cyan(),
                format!("  Max Clients :  {}", max_clients)
                    .bright_white()
                    .bold(),
                "│".bright_cyan()
            );

            if secret.is_some() {
                println!(
                    "  {}  {} {}",
                    "│".bright_cyan(),
                    "  Auth        :  ✔ Enabled".bright_green(),
                    "│".bright_cyan()
                );
            } else {
                println!(
                    "  {}  {} {}",
                    "│".bright_cyan(),
                    "  Auth        :  ✗ Disabled (open)".bright_yellow(),
                    "│".bright_cyan()
                );
            }

            println!(
                "  {}",
                "└──────────────────────────────────────────┘"
                    .bright_cyan()
            );
            println!(
                "\n  {} Waiting for connections... {}\n",
                "▶".bright_blue().bold(),
                "(Ctrl+C to stop)".dimmed()
            );

            let mut server = Server::new(port_range, secret.as_deref());
            server.set_bind_addr(bind_addr);
            server.set_bind_tunnels(bind_tunnels.unwrap_or(bind_addr));
            server.set_max_clients(max_clients);
            server.listen().await?;
        }
    }

    Ok(())
}

// ─── Entry Point ──────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    print_banner();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .compact()
        .init();
    run(Args::parse().command)
}
