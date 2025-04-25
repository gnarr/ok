use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{env, thread};
fn handle_connection(mut stream: TcpStream) {
    let timeout = Duration::from_secs(5);
    if let Err(e) = stream.set_read_timeout(Some(timeout)) {
        eprintln!("Failed to set read timeout: {}", e);
    }
    if let Err(e) = stream.set_write_timeout(Some(timeout)) {
        eprintln!("Failed to set write timeout: {}", e);
    }

    let _ = stream.read(&mut [0; 512]);

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
