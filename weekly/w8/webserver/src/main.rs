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
    // Convert the buffer into a readable String.
    let request = String::from_utf8_lossy(&buffer);

    // Load the HTML file as a byte array.
    let mut file = include_bytes!("../index.html").to_vec();
    // Placeholder to be replaced in the HTML content.
    let placeholder = "{{{ counter }}}";

    // Check if the request is a POST request to the "/counter" endpoint.
    if request.starts_with("POST /counter HTTP/1.1") {
        // Lock the state to increment the counter safely across threads.
        let mut state = state.lock().unwrap();
        state.counter += 1;
        // Convert the HTML file into a String and replace the placeholder with the current counter value.
        let file_str = String::from_utf8(file)
            .unwrap()
            .replace(placeholder, &state.counter.to_string());
        // Convert the modified HTML String back into a byte array.
        file = file_str.into_bytes();
    } else {
        // For other requests, just replace the placeholder without incrementing the counter.
        let state = state.lock().unwrap();
        let file_str = String::from_utf8(file)
            .unwrap()
            .replace(placeholder, &state.counter.to_string());
        file = file_str.into_bytes();
    }

    // DONT CHANGE ME
    let response = format!(
        "HTTP/1.1 200 OK\r\nContentType: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n",
        file.len()
    );

    // Write the response header and the modified HTML content back to the client.
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&file).unwrap();
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
