use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use cidr::{Ipv4Cidr, Ipv4Inet};
use futures::{stream, StreamExt};
use tokio::select;

/// Simple CLI port scanner over CIDR ranges
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// CIDR range to scan (e.g., 192.168.1.0/24)
    #[arg(short, long)]
    cidr: String,

    /// Ports to scan (e.g., 22,80,443 or 20~25)
    #[arg(short, long)]
    ports: String,

    // / Fast timeout for port checks (default: 30ms)
    #[arg(long, default_value = "30")]
    fast_timeout: u64,

    /// Slow timeout for port checks (default: 200ms)
    #[arg(long, default_value = "200")]
    slow_timeout: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkEndpoint {
    ip: Ipv4Addr,
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let ports = parse_ports(args.ports);

    let active_endpoints = scan_network_cidr(&args.cidr, ports, args.fast_timeout, args.slow_timeout).await;

    println!("Active network endpoints:");
    for ep in active_endpoints {
        println!("{}:{}", ep.ip, ep.port);
    }
}

async fn scan_network_cidr(cidr_input: &str, target_ports: Vec<u16>, fast_timeout: u64, slow_timeout: u64) -> Vec<NetworkEndpoint> {
    let ipv4_inet: Ipv4Inet = match cidr_input.parse() {
        Ok(parsed) => parsed,
        Err(_) => {
            eprintln!("Invalid CIDR notation: {}", cidr_input);
            return vec![];
        }
    };

    let network_cidr: Ipv4Cidr = ipv4_inet.network();
    let network_ips: Vec<Ipv4Addr> = network_cidr.into_iter().map(|inet| inet.address()).collect();

    let ports_clone = target_ports.clone();

    stream::iter(network_ips.into_iter().flat_map(move |ip| {
        let ports_per_ip = ports_clone.clone();
        ports_per_ip.into_iter().map(move |port| {
            let endpoint = NetworkEndpoint { ip, port };
            async move {
                if is_port_open(endpoint.clone(), fast_timeout, slow_timeout).await {
                    Some(endpoint)
                } else {
                    None
                }
            }
        })
    }))
    .buffer_unordered(1000)
    .filter_map(|result| async move { result })
    .collect()
    .await
}

async fn is_port_open(
    endpoint: NetworkEndpoint,
    fast_timeout: u64,
    slow_timeout: u64,
) -> bool {
    let socket_addr = SocketAddr::new(IpAddr::V4(endpoint.ip), endpoint.port);

    select! {
        connect_result = timeout(
            Duration::from_millis(fast_timeout),
            TcpStream::connect(socket_addr)
        ) => matches!(connect_result, Ok(Ok(_))),

        connect_result = timeout(
            Duration::from_millis(slow_timeout),
            TcpStream::connect(socket_addr)
        ) => matches!(connect_result, Ok(Ok(_))),

        else => false,
    }
}

fn parse_ports(input: String) -> Vec<u16> {
    let has_comma = input.contains(',');
    let has_range = input.contains('~');

    match (has_comma, has_range) {
        (true, false) => parse_comma_separated_ports(input),
        (false, true) => parse_port_range(input),
        (false, false) => parse_single_port(input),
        _ => {
            eprintln!("Invalid port input: cannot mix commas and range operators");
            vec![]
        }
    }
}

fn parse_comma_separated_ports(input: String) -> Vec<u16> {
    input.split(',').filter_map(|s| s.trim().parse::<u16>().ok()).collect()
}

fn parse_port_range(input: String) -> Vec<u16> {
    let mut ports = Vec::new();
    let parts: Vec<&str> = input.split('~').collect();

    if parts.len() == 2 {
        if let (Ok(start), Ok(end)) = (parts[0].trim().parse::<u16>(), parts[1].trim().parse::<u16>()) {
            if start <= end {
                ports.extend(start..=end);
            }
        }
    }
    ports
}

fn parse_single_port(input: String) -> Vec<u16> {
    match input.trim().parse() {
        Ok(port) => vec![port],
        Err(_) => {
            eprintln!("Invalid port number: {}", input);
            vec![]
        }
    }
}
