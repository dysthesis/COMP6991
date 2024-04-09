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
    // setup buffer, and read from stream into buffer
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // convert request payload to string
    let string = String::from_utf8_lossy(&buffer);

    // extract header
    let mut split = string.split("\r\n");
    let header = split.next().unwrap();

    let mut file = include_bytes!("../index.html").to_vec();
    let placeholder = "{{{ counter }}}";

    if header == "POST /counter HTTP/1.1" {
        let mut state = state.lock().unwrap();
        state.counter += 1;
        // Convert the HTML file into a String and replace the placeholder with the current counter value.
        let file_str = String::from_utf8(file)
            .unwrap()
            .replace(placeholder, &state.counter.to_string());
        // Convert the modified HTML String back into a byte array.
        file = file_str.into_bytes();
    } else {
        let state = state.lock().unwrap(); // Convert the HTML file into a String and replace the placeholder with the current counter value.
        let file_str = String::from_utf8(file)
            .unwrap()
            .replace(placeholder, &state.counter.to_string());
        // Convert the modified HTML String back into a byte array.
        file = file_str.into_bytes();
    }

    // TODO: replace triple brackets in file with the counter in state (array of bytes)
    //      - you should make sure your resulting content is still called file
    //      - or the below code will not work

    // DONT CHANGE ME
    let response = format!(
        "HTTP/1.1 200 OK\r\nContentType: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n",
        file.len()
    );

    // converts response to &[u8], and writes them to the stream
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&file).unwrap();
    stream.flush().unwrap();
}

fn main() -> std::io::Result<()> {
    let port = std::env::args().nth(1).unwrap_or("8081".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;

    println!("Server running on port {}", port);
    // TODO: create new state, so that it can be safely
    //      shared between threads
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State { counter: 0 }));

    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let state = Arc::clone(&state);
        thread::spawn(move || {
            handle_client(stream, state);
        });
    }
    Ok(())
}
