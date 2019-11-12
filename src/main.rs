use std::net::{TcpStream};
use std::io::{Read, Write, stdin};

fn main() {
  println!("Welcome to rs-gopher-client");

  loop {
    println!("Please enter command:");

    let mut command = String::new();
    stdin().read_line(&mut command)
      .expect("Failed to read line");

    command = String::from(command.trim());

    let mut server = String::new();
    if command.contains("go") {
      match command.get(3..) {
        Some(contains) => server = contains.to_string(),
        None => println!("Does not contain go"),
      }
    }
    else if command.contains("quit") {
      println!("Terminated.");
        break;
    }
    else {
      continue;
    }

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
  }
}
