use std::net::{TcpStream};
use std::io::{Read, Write, stdin};
use regex::Regex;

struct GopherURL {
  host: String,
  port: String,
  r#type: String,
  selector: String,
}

impl GopherURL {
  fn new() -> GopherURL {
    GopherURL {
      host: String::new(),
      port: String::new(),
      r#type: String::new(),
      selector: String::new(),
    }
  }

  fn from(url: &str) -> GopherURL {
    let re = Regex::new(r"(gopher://)?([a-zA-Z0-9.]+):?([0-9]+)?(/[a-z0-9]+)?([a-zA-Z0-9/.~\-_]+)?").unwrap();
    let captures = re.captures(url).unwrap();

    GopherURL {
      host: captures.get(2).map_or("", |m| m.as_str()).to_string(),
      port: captures.get(3).map_or("70", |m| m.as_str()).to_string(),
      r#type: captures.get(4).map_or("/1", |m| m.as_str())[1..].to_string(),
      selector: captures.get(5).map_or("/", |m| m.as_str()).to_string(),
    }
  }

  fn get_server(&self) -> String {
    return format!("{}:{}", &self.host, &self.port);
  }
}

struct GopherMenuLine {
  r#type: String,
  description: String,
  selector: String,
  host: String,
  port: String,
}

impl GopherMenuLine {
  fn new() -> GopherMenuLine {
    GopherMenuLine {
      r#type: String::new(),
      description: String::new(),
      selector: String::new(),
      host: String::new(),
      port: String::new(),
    }
  }

  fn from(line: &str) -> GopherMenuLine {
    let re = Regex::new(r"^([a-z0-9])([^\t]*)?\t([^\t]*)?\t([^\t]*)?\t([0-9]*)?").unwrap();
    let captures = re.captures(&line).unwrap();

    GopherMenuLine {
      r#type: captures.get(1).map_or("i", |m| m.as_str()).to_string(),
      description: captures.get(2).map_or("", |m| m.as_str()).to_string(),
      selector: captures.get(3).map_or("", |m| m.as_str()).to_string(),
      host: captures.get(4).map_or("", |m| m.as_str()).to_string(),
      port: captures.get(5).map_or("", |m| m.as_str()).to_string(),
    }
  }
}

struct GopherMenuResponse {
  lines: Vec<GopherMenuLine>,
}

impl GopherMenuResponse {
  fn new() -> GopherMenuResponse {
    GopherMenuResponse {
      lines: Vec::new(),
    }
  }

  fn from(response: &str) -> GopherMenuResponse {
    let mut lines = Vec::new();
    let mut gopherline = GopherMenuLine::new();

    for line in response.split("\n").collect::<Vec<&str>>() {
      // dot indicates end of response
      if line.starts_with(".") || line == "" {
        break;
      }

      gopherline = GopherMenuLine::from(line);
      lines.push(gopherline);
    }

    GopherMenuResponse {
      lines,
    }
  }

  fn display(&self) {
    for line in &self.lines {
      println!("{}{}\t{}\t{}\t{}", line.r#type, line.description, line.selector, line.host, line.port);
    }
  }
}

struct GopherDefaultResponse {
  lines: Vec<String>,
}

impl GopherDefaultResponse {
  fn new() -> GopherDefaultResponse {
    GopherDefaultResponse {
      lines: Vec::new(),
    }
  }

  fn from(response: &str) -> GopherDefaultResponse {
    let mut lines = Vec::new();

    for line in response.split("\n").collect::<Vec<&str>>() {
      // dot indicates end of response
      if line.starts_with(".") {
        break;
      }

      lines.push(line.to_string());
    }

    GopherDefaultResponse {
      lines,
    }
  }

  fn display(&self) {
    for line in &self.lines {
      println!("{}", line);
    }
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
    if command.contains("get") {
      match command.get(3..) {
        Some(contains) => {
          url = GopherURL::from(&contains);
        },
        None => println!("Does not contain get"),
      }
    }
    else if command.contains("quit") {
      println!("Terminated.");
      break;
    }
    else {
      println!("Please enter one of following command:\n\
                \tget [url]: Get to this url\n\
                \tquit: Quit this program");
      continue;
    }

    println!("\nConnecting to {}...", url.get_server());
    match TcpStream::connect(url.get_server()) {
      Ok(mut stream) => {
        println!("Connected!\n");

        stream.write(format!("{}\r\n", url.selector).as_bytes()).unwrap();

        let mut buffer = String::new();

        match stream.read_to_string(&mut buffer) {
          Ok(_) => {
            // Parse Gopher menu according to Gopher selector
            if url.r#type == "1" {
              let response = GopherMenuResponse::from(&buffer);
              response.display();
            }
            else {
              let response = GopherDefaultResponse::from(&buffer);
              response.display();
            }
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
