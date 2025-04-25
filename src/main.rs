use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{env, thread};

const MAX_HEADER_SIZE: usize = 8192;
fn read_request(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = Vec::new();
    let mut temp = [0; 512];

    loop {
        let n = stream.read(&mut temp)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..n]);

        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > MAX_HEADER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Header too large",
            ));
        }
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn handle_connection(mut stream: TcpStream) {
    let timeout = Duration::from_secs(5);
    let peer_address = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());

    stream
        .set_read_timeout(Some(timeout))
        .unwrap_or_else(|e| eprintln!("Error setting read timeout: {}", e));
    stream
        .set_write_timeout(Some(timeout))
        .unwrap_or_else(|e| eprintln!("Error setting write timeout: {}", e));

    let request = match read_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to read request: {}", e);
            return;
        }
    };

    let request_line = request.lines().next().unwrap_or("").to_string();
    println!(
        "{} \"{}\" {} bytes",
        peer_address,
        request_line,
        request.len()
    );

    let response = "\
HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Length: 2\r\n\
\r\n\
OK";

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write error: {}", e);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Flush error: {}", e);
    }
}

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&bind_addr)?;
    println!("Listening on {}", bind_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
