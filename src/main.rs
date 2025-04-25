use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use std::{env, thread};

const MAX_HEADER_SIZE: usize = 8192;

const OK_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
Content-Length: 2\r\n\r\n\
OK";

const FAVICON_HEADER: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: image/png\r\n\
Content-Length: 130\r\n\r\n";

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
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    let mut total_read = 0;
    let mut temp = [0u8; 512];

    let start_time = Instant::now();
    let max_duration = Duration::from_secs(5);

    loop {
        if start_time.elapsed() > max_duration {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Header read timeout",
            ));
        }

        let n = stream.read(&mut temp)?;
        if n == 0 {
            break;
        }
        if total_read + n > MAX_HEADER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Header too large",
            ));
        }
        buffer[total_read..total_read + n].copy_from_slice(&temp[..n]);
        total_read += n;

        let start = total_read.saturating_sub(n + 3);
        if buffer[start..total_read]
            .windows(4)
            .any(|w| w == b"\r\n\r\n")
        {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&buffer[..total_read]).to_string())
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
        let _ = stream.write_all(FAVICON_HEADER);
        let _ = stream.write_all(FAVICON_PNG);
        let _ = stream.flush();
        return;
    }
    let _ = stream.write_all(OK_RESPONSE);
    let _ = stream.flush();
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
        thread::spawn(move || {
            loop {
                let stream = thread_receiver.lock().unwrap().recv().unwrap();
                handle_connection(stream);
            }
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
