use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::panic;
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::time::{Duration, Instant};
use std::{env, thread};

const MAX_HEADER_SIZE: usize = 8192;
const MAX_BODY_SIZE: usize = 1 * 1024 * 1024;
const QUEUE_CAPACITY: usize = 100;
const LOG_QUEUE_CAPACITY: usize = 100;

const OK_HEADER: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 2\r\n\r\n";
const OK_BODY: &[u8] = b"OK";

const FAVICON_HEADER: &[u8] = b"HTTP/1.1 200 OK\r\n\
Connection: close\r\n\
Content-Type: image/png\r\n\
Cache-Control: public, max-age=86400\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 130\r\n\r\n";

const RESPONSE_404: &[u8] = b"HTTP/1.1 404 Not Found\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n\
Content-Length: 0\r\n\r\n";
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
        .map(|c| if c.is_control() || c == '"' { '?' } else { c })
        .collect()
}

fn parse_request_line(request_line: &str) -> (&str, &str) {
    if let Some(method_end_index) = request_line.find(' ') {
        let method = &request_line[..method_end_index];
        let rest = &request_line[method_end_index + 1..];
        if let Some(path_end_index) = rest.find(' ') {
            let path = &rest[..path_end_index];
            return (method, path);
        }
        return (method, rest);
    }
    ("", "")
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed before full header was received",
            ));
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

fn get_client_address(stream: &mut TcpStream, headers: &str) -> String {
    for line in headers.lines() {
        if let Some(val) = line
            .strip_prefix("X-Forwarded-For:")
            .or_else(|| line.strip_prefix("x-forwarded-for:"))
        {
            return sanitize(val.trim().split(',').next().unwrap_or(""));
        }
    }
    stream
        .peer_addr()
        .map(|a| sanitize(&a.to_string()))
        .unwrap_or_else(|_| "unknown".into())
}

fn handle_connection(mut stream: TcpStream, log_tx: SyncSender<String>, show_favicon: bool) {
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
        if line.len() >= 15 && line[..15].eq_ignore_ascii_case("content-length:") {
            let val = &line[15..];
            if content_length != 0 {
                let _ = stream.write_all(RESPONSE_431);
                return;
            }
            match val.trim().parse::<usize>() {
                Ok(len) => content_length = len,
                Err(_) => {
                    let _ = stream.write_all(RESPONSE_431);
                    return;
                }
            }
        }
    }

    let peer = get_client_address(&mut stream, &headers);
    let request_line = headers.lines().next().unwrap_or("");
    let byte_count = headers.len() + content_length;
    let log_message = format!(
        "{} \"{}\" {} bytes",
        peer,
        sanitize(request_line),
        byte_count
    );
    match log_tx.try_send(log_message) {
        Ok(_) => {}
        Err(TrySendError::Full(_)) => {}
        Err(TrySendError::Disconnected(_)) => {
            eprintln!("log channel disconnected – log entry lost");
        }
    }

    let (method, path) = parse_request_line(request_line);

    match (method, path) {
        (m @ ("GET" | "HEAD"), "/") => {
            let _ = stream.write_all(OK_HEADER);
            if m == "GET" {
                let _ = stream.write_all(OK_BODY);
            }
        }
        (m @ ("GET" | "HEAD"), "/favicon.ico") if show_favicon => {
            let _ = stream.write_all(FAVICON_HEADER);
            if m == "GET" {
                let _ = stream.write_all(FAVICON_PNG);
            }
        }
        ("HEAD", _) => {
            let _ = stream.write_all(RESPONSE_404);
        }
        (method, _) if method != "GET" && method != "HEAD" => {
            let _ = stream.write_all(RESPONSE_501);
        }
        _ => {
            if content_length > 0 {
                if let Err(e) = read_body(&mut stream, content_length) {
                    if e.kind() == std::io::ErrorKind::InvalidData {
                        let _ = stream.write_all(RESPONSE_413);
                    }
                    return;
                }
            }
            let _ = stream.write_all(RESPONSE_404);
        }
    }
    let _ = stream.flush();
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    if args.len() == 2 && args.nth(1).as_deref() == Some("--health-check") {
        std::process::exit(0);
    }
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let bind_addr = format!("0.0.0.0:{}", port);
    let show_favicon = env::var("SHOW_FAVICON")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    let pool_size = env::var("THREAD_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });

    let listener = TcpListener::bind(&bind_addr)?;
    println!(
        "Listening on {} with {} worker threads",
        bind_addr, pool_size
    );

    let (log_tx, log_rx) = sync_channel::<String>(LOG_QUEUE_CAPACITY);
    thread::spawn(move || {
        for msg in log_rx {
            println!("{}", msg);
        }
    });

    let mut senders = Vec::with_capacity(pool_size);
    for _ in 0..pool_size {
        let (tx, rx) = sync_channel::<TcpStream>(QUEUE_CAPACITY);
        senders.push(tx.clone());
        let log_tx_clone = log_tx.clone();
        let show_favicon = show_favicon;
        thread::spawn(move || {
            for stream in rx {
                if let Err(err) = panic::catch_unwind(|| {
                    handle_connection(stream, log_tx_clone.clone(), show_favicon)
                }) {
                    eprintln!("Worker thread panicked: {:?}", err);
                }
            }
        });
    }

    let mut next = 0;
    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let tx = &senders[next];
            next = (next + 1) % pool_size;

            match tx.try_send(stream) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {
                    let _ = log_tx.try_send("Connection dropped: worker queue is full".into());
                }
                Err(TrySendError::Disconnected(_)) => {
                    let _ =
                        log_tx.try_send("Worker queue disconnected – dropping connection".into());
                }
            }
        }
    }
    Ok(())
}
