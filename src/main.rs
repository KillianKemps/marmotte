use std::net::{TcpStream};
use std::io::{Read, Write, stdin};

fn main() {
  println!("Welcome to rs-gopher-client");
  println!("Please enter server:");

  let mut server = String::new();
  stdin().read_line(&mut server)
    .expect("Failed to read line");

  server = server.trim().to_string();

  println!("\nConnecting to {}...", server);
  match TcpStream::connect(format!("{}:70", server)) {
    Ok(mut stream) => {
      println!("Connected!\n");

      let msg = b"/\r\n";

      stream.write(msg).unwrap();

      let mut buffer = String::new();

      match stream.read_to_string(&mut buffer) {
        Ok(_) => {
          println!("{}", &buffer);
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
