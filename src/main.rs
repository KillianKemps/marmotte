use std::net::{TcpStream};
use std::io::{Read, Write, stdin};
use regex::Regex;

struct GopherURL {
  host: String,
  port: String,
  path: String,
}

impl GopherURL {
  fn new() -> GopherURL {
    GopherURL {
      host: "".to_string(),
      port: "".to_string(),
      path: "".to_string(),
    }
  }

  fn from(url: &str) -> GopherURL {
    let re = Regex::new(r"(gopher://)?([a-zA-Z0-9.]+):?([0-9]+)?([a-zA-Z0-9/.]+)?").unwrap();
    let captures = re.captures(url).unwrap();

    GopherURL {
      host: captures.get(2).map_or("", |m| m.as_str()).to_string(),
      port: captures.get(3).map_or("70", |m| m.as_str()).to_string(),
      path: captures.get(4).map_or("/", |m| m.as_str()).to_string(),
    }
  }

  fn get_server(&self) -> String {
    return format!("{}:{}", &self.host, &self.port);
  }
}

fn main() {
  println!("Welcome to rs-gopher-client");

  loop {
    println!("Please enter command:");

    let mut command = String::new();
    stdin().read_line(&mut command)
      .expect("Failed to read line");

    command = String::from(command.trim());

    let mut url = GopherURL::new();
    if command.contains("go") {
      match command.get(3..) {
        Some(contains) => {
          url = GopherURL::from(&contains);
        },
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

    println!("\nConnecting to {}...", url.get_server());
    match TcpStream::connect(url.get_server()) {
      Ok(mut stream) => {
        println!("Connected!\n");

        stream.write(format!("{}\r\n", url.path).as_bytes()).unwrap();

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
