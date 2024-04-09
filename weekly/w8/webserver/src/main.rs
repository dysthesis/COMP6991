//! A simple web server
//! which serves some html at index.html
//! and replaces triple curly braces with the given variable
mod test;
use std::io::{Read, Write};

use std::net::{TcpListener, TcpStream};
// hint, hint
use std::sync::{Arc, Mutex};
use std::thread;

struct State {
    counter: i32,
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<State>>) {
    // Buffer to store incoming data from the stream.
    let mut buffer = [0; 1024];
    // Read data from the stream into the buffer.
    stream.read(&mut buffer).unwrap();
    // Determine the type of request (GET or POST) and the requested path.
    let binding = String::from_utf8_lossy(&buffer);
    let request_line = binding.lines().next().unwrap_or_default();
    let mut counter_value = String::new();

    {
        // Lock the state to safely access the counter. The lock is scoped to allow it to be released before writing to the stream.
        let mut state = state.lock().unwrap();

        if request_line.starts_with("POST /counter") {
            // Increment the counter for POST requests to "/counter".
            state.counter += 1;
        }

        // Copy the counter value to a string to be used in the HTML replacement.
        counter_value = state.counter.to_string();
    } // MutexGuard is dropped here, releasing the lock.

    // Load and modify the HTML content.
    let html_template = include_str!("../index.html");
    let html_content = html_template.replace("{{{ counter }}}", &counter_value);

    // Construct the HTTP response with the modified HTML content. Keeping the 'DONT CHANGE ME' part intact.
    // DONT CHANGE ME
    let response = format!(
        "HTTP/1.1 200 OK\r\nContentType: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        html_content.len(),
        html_content
    );

    // Write the HTTP response back to the client.
    stream.write_all(response.as_bytes()).unwrap();
    // Ensure all data is sent before closing the connection.
    stream.flush().unwrap();
}

fn main() -> std::io::Result<()> {
    let port = std::env::args().nth(1).unwrap_or("8081".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;

    println!("Server running on port {}", port);
    // TODO: create new state, so that it can be safely
    //      shared between threads
    // Initialize shared state with a counter set to 0, wrapped in Arc and Mutex for safe sharing across threads.
    let state = Arc::new(Mutex::new(State { counter: 0 }));

    // Listen for incoming TCP connections in a loop.
    for stream in listener.incoming() {
        let stream = stream?;
        // Clone the Arc to retain shared ownership of the state across threads.
        let state = Arc::clone(&state);

        // Spawn a new thread for each connection, handling it in isolation.
        thread::spawn(move || {
            handle_client(stream, state);
        });
    }
    Ok(())
}
