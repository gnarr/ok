use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

fn handle_connection(mut stream: TcpStream) {
    // Read the request (we don't care about its content)
    let _ = stream.read(&mut [0; 512]);

    // Prepare a simple HTTP 200 OK response with body "OK"
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
    let _ = stream.write(response.as_bytes());
}

fn main() -> std::io::Result<()> {
    // Read port from environment variable PORT, default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    // Bind to all interfaces on the specified port
    let listener = TcpListener::bind(&bind_addr)?;
    println!("Listening on {}", bind_addr);

    // Handle each incoming connection
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}