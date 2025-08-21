use std::env;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <hostname> <port>", args[0]);
        eprintln!("Example: {} google.com 80", args[0]);
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
    
    println!("Checking {}:{}", hostname, port);
    
    match check_port(hostname, port) {
        Ok(()) => {
            println!("✓ Connection to {}:{} succeeded - port is open", hostname, port);
            process::exit(0);
        }
        Err(e) => {
            println!("✗ Connection to {}:{} failed - {}", hostname, port, e);
            process::exit(1);
        }
    }
}

fn check_port(hostname: &str, port: u16) -> Result<(), String> {
    let address = format!("{}:{}", hostname, port);
    
    // Resolve hostname to socket addresses
    let socket_addrs: Vec<_> = address.to_socket_addrs()
        .map_err(|e| format!("Failed to resolve hostname '{}': {}", hostname, e))?
        .collect();
    
    if socket_addrs.is_empty() {
        return Err(format!("No addresses found for hostname '{}'", hostname));
    }
    
    // Try to connect with a reasonable timeout
    let timeout = Duration::from_secs(5);
    
    for (i, addr) in socket_addrs.iter().enumerate() {
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(_stream) => {
                // Connection successful, port is open
                return Ok(());
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
