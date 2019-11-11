use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
  println!("Welcome to rs-gopher-client");
  let server = "khzae.net";
  match TcpStream::connect(format!("{}:70", server)) {
    Ok(mut stream) => {
      println!("Connected to {}!", server);

      let msg = b"/\r\n";

      stream.write(msg).unwrap();
      println!("Sent /, awaiting reply...");

      //let mut buffer = String::new();
      let mut buffer = Vec::new();

      match stream.read_to_end(&mut buffer) {
        Ok(_) => {
          let text = from_utf8(&buffer).unwrap();
          println!("response: {}", text);
        },
        Err(e) => {
          println!("Failed to receive data: {}", e);
        }
      }
    },

    Err(e) => {
      println!("Failed to connect: {}", e);
    }
  }
  println!("Terminated.");
}
