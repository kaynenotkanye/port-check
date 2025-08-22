use std::env;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let hostname = &args[1];
    let port_str = &args[2];

    let port: u16 = match port_str.parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Error: Invalid port number '{}'", port_str);
            process::exit(1);
        }
    };

    // Parse timeout option (default is 5 seconds)
    let timeout_secs = if args.len() >= 4 && args[3] == "--timeout" {
        if args.len() != 5 {
            eprintln!("Error: --timeout requires a value");
            print_usage(&args[0]);
            process::exit(1);
        }
        match args[4].parse::<u64>() {
            Ok(t) if t > 0 => t,
            _ => {
                eprintln!("Error: Timeout must be a positive number");
                process::exit(1);
            }
        }
    } else if args.len() == 3 {
        5 // default timeout
    } else {
        eprintln!("Error: Invalid arguments");
        print_usage(&args[0]);
        process::exit(1);
    };

    // Resolve hostname first to show IP in the checking message
    let address = format!("{}:{}", hostname, port);
    let socket_addrs: Vec<_> = match address.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("✗ Failed to resolve hostname '{}': {}", hostname, e);
            process::exit(1);
        }
    };

    if socket_addrs.is_empty() {
        eprintln!("✗ No addresses found for hostname '{}'", hostname);
        process::exit(1);
    }

    let primary_ip = socket_addrs[0].ip();
    println!("Checking {}:{} ({}) (timeout: {}s)", hostname, port, primary_ip, timeout_secs);

    match check_port_with_addrs(&socket_addrs, timeout_secs) {
        Ok(resolved_addr) => {
            println!("✓ Connection to {}:{} ({}) succeeded - port is open",
                     hostname, port, resolved_addr);
            process::exit(0);
        }
        Err(e) => {
            println!("✗ Connection to {}:{} ({}) failed - {}", hostname, port, primary_ip, e);
            process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <hostname> <port> [--timeout <seconds>]", program_name);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <hostname>  The hostname or IP address to check");
    eprintln!("  <port>      The TCP port number to test");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --timeout <seconds>  Connection timeout in seconds (default: 5)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} google.com 80", program_name);
    eprintln!("  {} google.com 443 --timeout 10", program_name);
    eprintln!("  {} localhost 22 --timeout 1", program_name);
    eprintln!("  {} 192.168.1.1 3389 --timeout 15", program_name);
}

fn check_port_with_addrs(socket_addrs: &[std::net::SocketAddr], timeout_secs: u64) -> Result<std::net::IpAddr, String> {
    // Use the specified timeout
    let timeout = Duration::from_secs(timeout_secs);

    for (i, addr) in socket_addrs.iter().enumerate() {
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(_stream) => {
                // Connection successful, return the IP address
                return Ok(addr.ip());
            }
            Err(e) => {
                // If this is the last address and it failed, return the error
                if i == socket_addrs.len() - 1 {
                    return Err(format!("port closed or unreachable ({})", e));
                }
                // Otherwise, try the next address
                continue;
            }
        }
    }

    Err("All connection attempts failed".to_string())
}