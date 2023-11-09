use std::env;
use std::convert::Infallible;
use std::fmt::Display;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::path::PathBuf;

use tokio::fs;
use tokio::process;

use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use askama::Template;
use clap::{Parser, Subcommand};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;


const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

static DEFAULT_LISTEN_ADDR: &str = "[::]:8080"; // listen for IPv4 and IPv6

static DEFAULT_ASN: &str = "AS63806";
static DEFAULT_AS_NAME: &str = "MENHERA";
static DEFAULT_ROUTER_DOMAIN: &str = "nc.menhera.org";
static DEFAULT_ROUTER_ID: &str = "rv128";
static DEFAULT_ADDRESS_V4: &str = "43.228.174.128";
static DEFAULT_ADDRESS_V6: &str = "2001:df3:14c0:1128::1";

static INSTALL_DIR: &str = "/opt/router-hello";

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(about = "Tiny HTTP server which displays a router's information", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Installs router-hello using systemd
    Install,
    /// Runs a HTTP server
    Http,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum RemoteAddr {
    V4(String),
    V6(String),
}

impl Display for RemoteAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(v4_addr_str) => {
                f.write_str(&format!("IPv4({})", v4_addr_str))
            }
            Self::V6(v6_addr_str) => {
                f.write_str(&format!("IPv6({})", v6_addr_str))
            }
        }
    }
}

impl From<IpAddr> for RemoteAddr {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(v4_addr) => {
                Self::V4(v4_addr.to_string())
            }
            IpAddr::V6(v6_addr) => {
                if let Some(v4_addr) = v6_addr.to_ipv4_mapped() {
                    Self::V4(v4_addr.to_string())
                } else {
                    Self::V6(v6_addr.to_string())
                }
            }
        }
    }
}

#[derive(Template, Clone, Debug)]
#[template(path = "index.html")]
struct IndexTemplate {
    pkg_name: String,
    pkg_version: String,
    asn: String,
    as_name: String,
    router_domain: String,
    router_id: String,
    address_v4: String,
    address_v6: String,
    remote_address: RemoteAddr,
}

impl IndexTemplate {
    fn new(remote_address: RemoteAddr) -> Self {
        let asn = env::var("ASN").unwrap_or(DEFAULT_ASN.to_string());
        let as_name = env::var("AS_NAME").unwrap_or(DEFAULT_AS_NAME.to_string());
        let router_domain = env::var("ROUTER_DOMAIN").unwrap_or(DEFAULT_ROUTER_DOMAIN.to_string());
        let router_id = env::var("ROUTER_ID").unwrap_or(DEFAULT_ROUTER_ID.to_string());
        let address_v4 = env::var("ADDRESS_V4").unwrap_or(DEFAULT_ADDRESS_V4.to_string());
        let address_v6 = env::var("ADDRESS_V6").unwrap_or(DEFAULT_ADDRESS_V6.to_string());
        IndexTemplate {
            pkg_name: PKG_NAME.to_string(),
            pkg_version: PKG_VERSION.to_string(),
            asn,
            as_name,
            router_domain,
            router_id,
            address_v4,
            address_v6,
            remote_address,
        }
    }
}

async fn http() {
    let addr_string = env::var("LISTEN_ADDR").unwrap_or("".to_string());
    let addr = SocketAddr::from_str(&addr_string).unwrap_or(SocketAddr::from_str(DEFAULT_LISTEN_ADDR).unwrap());

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        async move {
            let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |_req : Request<Body>| {
                let remote_address: RemoteAddr = addr.ip().into();
                let template = IndexTemplate::new(remote_address);
                let res = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body(Body::from(template.render().unwrap()))
                    .unwrap();
                async move {
                    Ok::<_, Infallible>(res)
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

fn readline(prompt: &str, default: &str) -> String {
    let mut rl = DefaultEditor::new().unwrap();
    let prompt = format!("{} [{}]: ", prompt, default);
    let res = rl.readline(&prompt);
    match res {
        Ok(line) => {
            let line = line.trim();
            if line.is_empty() {
                default.to_string()
            } else {
                line.to_string()
            }
        }
        Err(ReadlineError::Interrupted) => {
            panic!("Aborting.");
        }
        Err(ReadlineError::Eof) => {
            panic!("EOF reached. Aborting.");
        }
        Err(err) => {
            panic!("Error: {:?}", err);
        }
    }
}

async fn install() {
    println!("Starting configuration.");
    let listen_addr = readline("HTTP Listen Address", "[::]:80");
    let as_number = readline("AS number", "AS63806");
    let as_name = readline("AS name", "MENHERA");
    let router_domain = readline("Router domain (router IDs are subdomains of this domain)", "nc.menhera.org");
    let router_id = readline("Router ID (hostname of router)", "rv128");
    let address_v4 = readline("Router IPv4 address", "43.228.174.128");
    let address_v6 = readline("Router IPv6 address", "2001:df3:14c0:1128::1");
    println!("");
    let env_path = "/etc/default/router-hello";
    println!("Installing configuration...");
    let config = format!("
LISTEN_ADDR={listen_addr}
ASN={as_number}
AS_NAME={as_name}
ROUTER_DOMAIN={router_domain}
ROUTER_ID={router_id}
ADDRESS_V4={address_v4}
ADDRESS_V6={address_v6}
");

    println!("Configuration:\n\n{config}\n");
    fs::write(env_path, config).await.unwrap();
    println!("You can change settings later in {}", env_path);
    println!("");

    println!("Installing router-hello into: {}", INSTALL_DIR);
    let install_dir = PathBuf::from_str(INSTALL_DIR).unwrap();
    fs::create_dir_all(&install_dir).await.unwrap();
    println!("Getting the current binary path...");
    let binary_path = fs::canonicalize(std::env::current_exe().unwrap()).await.unwrap();
    println!("Current binary: {}", binary_path.display());
    let bin_dir = install_dir.join("bin");
    let binary_install_path = bin_dir.join("router-hello");
    println!("Installing binary to: {}", binary_install_path.display());

    let mut systemd_stop_command = process::Command::new("systemctl")
        .arg("stop")
        .arg("router-hello")
        .spawn()
        .expect("Failed to execute systemctl");
    let status = systemd_stop_command.wait().await.unwrap();
    if status.success() {
        println!("Stopped running router-hello service.");
    }

    fs::create_dir_all(&bin_dir).await.unwrap();
    fs::copy(&binary_path, &binary_install_path).await.unwrap();

    let systemd_service_str = include_str!("router-hello.service");
    let systemd_dir = install_dir.join("lib/systemd/system");
    let systemd_service_path = systemd_dir.join("router-hello.service");
    println!("Installing systemd service file...");
    fs::create_dir_all(systemd_dir).await.unwrap();
    fs::write(&systemd_service_path, systemd_service_str).await.unwrap();
    let systemd_service_install_path = "/etc/systemd/system/router-hello.service";
    println!("Installing service file: {}", systemd_service_install_path);
    if let Ok(_) = fs::remove_file(systemd_service_install_path).await {
        println!("Removed the previous service file.");
    };
    fs::symlink(&systemd_service_path, systemd_service_install_path).await.unwrap();

    println!("Starting router-hello service...");
    let mut systemd_daemon_reload_command = process::Command::new("systemctl")
        .arg("daemon-reload")
        .spawn()
        .expect("Failed to execute systemctl");
    let status = systemd_daemon_reload_command.wait().await.unwrap();
    if !status.success() {
        panic!("systemctl daemon-reload failed");
    }

    let mut systemd_restart_command = process::Command::new("systemctl")
        .arg("restart")
        .arg("router-hello")
        .spawn()
        .expect("Failed to execute systemctl");
    let status = systemd_restart_command.wait().await.unwrap();
    if !status.success() {
        panic!("systemctl restart router-hello failed");
    }
    println!("Started router-hello service.");
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Http => {
            http().await;
        }

        Commands::Install => {
            install().await;
        }
    };
}
