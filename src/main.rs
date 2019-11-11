use std::net::{TcpStream};
use std::io::{Read, Write};

fn main() {
  println!("Welcome to rs-gopher-client");
  let server = "khzae.net";
  match TcpStream::connect(format!("{}:70", server)) {
    Ok(mut stream) => {
      println!("Connected to {}!", server);

      let msg = b"/\r\n";

      stream.write(msg).unwrap();
      println!("Sent /, awaiting reply...");

      let mut buffer = String::new();

      match stream.read_to_string(&mut buffer) {
        Ok(_) => {
          println!("response:\n{}", &buffer);
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
