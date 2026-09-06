use anyhow::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn serve(port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("AIORG server listening on http://127.0.0.1:{port}");
    for stream in listener.incoming().flatten() {
        thread::spawn(|| {
            let _ = handle(stream);
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0u8; 8192];
    let n = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let (content_type, body) = match path {
        "/events" => ("text/event-stream", "event: ready\ndata: {\"status\":\"operational\"}\n\n".to_string()),
        "/mcp" => ("application/json", "{\"jsonrpc\":\"2.0\",\"result\":{\"tools\":[\"aiorg_run\",\"aiorg_status\",\"aiorg_certificate_verify\"]}}".to_string()),
        "/status" | "/" => ("application/json", "{\"status\":\"operational\",\"transport\":\"local\"}".to_string()),
        _ => ("application/json", "{\"error\":\"not_found\"}".to_string()),
    };
    let response=format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nCache-Control: no-cache\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
    stream.write_all(response.as_bytes())?;
    Ok(())
}
