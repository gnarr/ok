use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{env, thread};
use std::sync::{mpsc, Arc, Mutex};

const MAX_HEADER_SIZE: usize = 8192;

const FAVICON_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x08, 0x03, 0x00, 0x00, 0x00, 0x28, 0x2D, 0x0F,
    0x53, 0x00, 0x00, 0x00, 0x06, 0x50, 0x4C, 0x54, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA5,
    0x67, 0xB9, 0xCF, 0x00, 0x00, 0x00, 0x01, 0x74, 0x52, 0x4E, 0x53, 0x00, 0x40, 0xE6, 0xD8, 0x66,
    0x00, 0x00, 0x00, 0x2A, 0x49, 0x44, 0x41, 0x54, 0x18, 0xD3, 0x63, 0x60, 0xA0, 0x0E, 0x60, 0x64,
    0x64, 0x04, 0x93, 0x10, 0x0C, 0x26, 0xA1, 0x1C, 0x08, 0x9F, 0x81, 0x11, 0x26, 0x08, 0xE3, 0x13,
    0x14, 0x40, 0x68, 0x61, 0x40, 0x28, 0x41, 0x35, 0x14, 0xC3, 0x5A, 0x8A, 0x01, 0x00, 0x20, 0xDE,
    0x00, 0x3D, 0xEB, 0xB1, 0x31, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42,
    0x60, 0x82,
];
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

fn write_response(stream: &mut TcpStream, content_type: &str, content: &[u8]) {
    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
             Connection: close\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\r\n",
        content_type,
        content.len()
    );
    if let Err(e) = stream.write_all(headers.as_bytes()) {
        eprintln!("Write headers error: {}", e);
        return
    }
    if let Err(e) = stream.write_all(content) {
        eprintln!("Write data error: {}", e);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Flush error: {}", e);
    }
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

    if request_line.starts_with("GET /favicon.ico") {
        write_response(&mut stream, "image/png", FAVICON_PNG);
        return;
    }

    write_response(&mut stream, "text/plain", b"OK");
}

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&bind_addr)?;
    println!("Listening on {}", bind_addr);

    let (sender, receiver) = mpsc::channel::<TcpStream>();
    let receiver = Arc::new(Mutex::new(receiver));
    let pool_size = 4;
    for _ in 0..pool_size {
        let thread_receiver = Arc::clone(&receiver);
        thread::spawn(move || loop {
            let stream = thread_receiver.lock().unwrap().recv().unwrap();
            handle_connection(stream);
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if sender.send(stream).is_err() {
                    eprintln!("Worker threads have shut down");
                    break;
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
