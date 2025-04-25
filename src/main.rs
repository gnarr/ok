use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::panic;
use std::sync::mpsc::{TrySendError, sync_channel, Sender, channel};
use std::time::{Duration, Instant};
use std::{env, thread};

const MAX_HEADER_SIZE: usize = 8192;
const MAX_BODY_SIZE: usize = 1 * 1024 * 1024;
const QUEUE_CAPACITY: usize = 100;
const POOL_SIZE: usize = 4;

const OK_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 2\r\n\r\n\
OK";

const FAVICON_HEADER: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: image/png\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 130\r\n\r\n";

const RESPONSE_408: &[u8] = b"HTTP/1.1 408 Request Timeout\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 0\r\n\r\n";
const RESPONSE_431: &[u8] = b"HTTP/1.1 431 Request Header Fields Too Large\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 0\r\n\r\n";
const RESPONSE_413: &[u8] = b"HTTP/1.1 413 Payload Too Large\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 0\r\n\r\n";
const RESPONSE_501: &[u8] = b"HTTP/1.1 501 Not Implemented\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 0\r\n\r\n";

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

fn sanitize(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_control() || c == '"' {
                '?' // replace with placeholder
            } else {
                c
            }
        })
        .collect()
}

fn read_headers(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    let mut total_read = 0;
    let mut temp = [0u8; 512];

    let start_time = Instant::now();
    let deadline = Duration::from_secs(5);

    loop {
        if start_time.elapsed() > deadline {
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

fn read_body(stream: &mut TcpStream, mut remaining: usize) -> std::io::Result<()> {
    if remaining > MAX_BODY_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Body too large",
        ));
    }
    let mut buf = [0u8; 4096];
    while remaining > 0 {
        let to_read = std::cmp::min(buf.len(), remaining);
        let n = stream.read(&mut buf[..to_read])?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed before full body was received",
            ));
        }
        remaining -= n;
    }
    Ok(())
}

fn get_client_address(stream: &mut TcpStream, headers: &String) -> String {
    let mut client_ip: Option<String> = None;
    for line in headers.lines() {
        if let Some(val) = line
            .strip_prefix("X-Forwarded-For:")
            .or_else(|| line.strip_prefix("x-forwarded-for:"))
        {
            client_ip = Some(val.trim().split(',').next().unwrap_or("").to_string());
            break;
        }
    }
    let fallback_ip = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());
    let peer_address_raw = client_ip.unwrap_or(fallback_ip);
    let peer_address = sanitize(&peer_address_raw);
    peer_address
}

fn handle_connection(mut stream: TcpStream, log_tx: Sender<String>) {
    let timeout = Duration::from_secs(5);
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();

    let headers = match read_headers(&mut stream) {
        Ok(h) => h,
        Err(e) => match e.kind() {
            std::io::ErrorKind::TimedOut => {
                let _ = stream.write_all(RESPONSE_408);
                return;
            }
            std::io::ErrorKind::InvalidData => {
                let _ = stream.write_all(RESPONSE_431);
                return;
            }
            _ => return,
        },
    };

    for line in headers.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("transfer-encoding:") && lower.contains("chunked") {
            let _ = stream.write_all(RESPONSE_501);
            return;
        }
    }

    let mut content_length = 0;
    for line in headers.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(val) = lower.strip_prefix("content-length:") {
            if content_length != 0 {
                // duplicated header ➜ 400
                let _ = stream.write_all(RESPONSE_431);
                return;
            }
            if let Ok(len) = val.trim().parse::<usize>() {
                content_length = len;
            } else {
                // malformed value ➜ 400
                let _ = stream.write_all(RESPONSE_431);
                return;
            }
        }
    }

    let peer_address = get_client_address(&mut stream, &headers);
    let request_line_raw = headers.lines().next().unwrap_or("");
    let request_line = sanitize(request_line_raw);
    let byte_count = headers.len() + content_length;

    let log_message = format!("{} \"{}\" {} bytes", peer_address, request_line, byte_count);
    let _ = log_tx.send(log_message);

    if content_length > 0 {
        if let Err(e) = read_body(&mut stream, content_length) {
            if e.kind() == std::io::ErrorKind::InvalidData {
                let _ = stream.write_all(RESPONSE_413);
            }
            return;
        }
    }

    if request_line.starts_with("GET /favicon.ico") {
        let _ = stream.write_all(FAVICON_HEADER);
        let _ = stream.write_all(FAVICON_PNG);
    } else {
        let _ = stream.write_all(OK_RESPONSE);
    }
    let _ = stream.flush();
}

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&bind_addr)?;
    println!("Listening on {}", bind_addr);

    let (log_tx, log_rx) = channel::<String>();
    thread::spawn(move || {
        for msg in log_rx {
            println!("{}", msg);
        }
    });

    let mut senders = Vec::with_capacity(POOL_SIZE);
    for _ in 0..POOL_SIZE {
        let (tx, rx) = sync_channel::<TcpStream>(QUEUE_CAPACITY);
        senders.push(tx);
        let log_tx_clone = log_tx.clone();
        thread::spawn(move || {
            for stream in rx {
                if let Err(err) = panic::catch_unwind(|| handle_connection(stream, log_tx_clone.clone())) {
                    eprintln!("Worker thread panicked: {:?}", err);
                }
            }
        });
    }

    let mut next = 0;
    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let tx = &senders[next];
            next = (next + 1) % POOL_SIZE;
            if let Err(TrySendError::Full(_)) = tx.try_send(stream) {
                let _ = log_tx.send(format!("Connection dropped: worker queue is full"));
            }
        }
    }
    Ok(())
}
