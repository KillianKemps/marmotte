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

  fn get_url(&self) -> Option<String> {
    if &self.host == "" {
      return None;
    }
    else {
      return Some(format!("{}:{}/{}{}", &self.host, &self.port, &self.r#type, &self.selector));
    }
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
  #[allow(dead_code)]
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

  fn get_url(&self) -> String {
    if &self.host == "" {
      return String::new();
    }
    else {
      return format!("{}:{}/{}{}", &self.host, &self.port, &self.r#type, &self.selector);
    }
  }
}

struct GopherMenuResponse {
  lines: Vec<GopherMenuLine>,
}

impl GopherMenuResponse {
  #[allow(dead_code)]
  fn new() -> GopherMenuResponse {
    GopherMenuResponse {
      lines: Vec::new(),
    }
  }

  fn from(response: &str) -> GopherMenuResponse {
    let mut lines = Vec::new();

    for line in response.split("\r\n").collect::<Vec<&str>>() {
      // dot indicates end of response
      if line.starts_with(".") || line.trim() == "" {
        break;
      }

      let gopherline = GopherMenuLine::from(line);
      lines.push(gopherline);
    }

    GopherMenuResponse {
      lines,
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
}

enum GopherResponse {
  Default(GopherDefaultResponse),
  Menu(GopherMenuResponse),
}

impl GopherResponse {
  fn display(&self) {
    match &self {
      GopherResponse::Default(response) => {
        for line in &response.lines {
          println!("{}", line);
        }
      },
      GopherResponse::Menu(response) => {
        for (index, line) in response.lines.iter().enumerate() {
          match &line.r#type[..] {
            "0" => {
              let resource_type = "TXT";
              println!("{}\t[{}]\t{}", resource_type, index, line.description);
            },
            "1" => {
              let resource_type = "MENU";
              println!("{}\t[{}]\t{}/", resource_type, index, line.description);
            },
            _ => {
              if line.r#type != "i" {
                let resource_type = "OTHER";
                println!("{}\t[{}]\t{}", resource_type, index, line.description);
              }
              else {
                println!("\t\t{}", line.description);
              }
            },
          }
        }
      }
    }
  }

  fn get_link_url(&self, link_idx: &str) -> Option<String> {
    let index:usize = link_idx.parse().unwrap();
    match &self {
      GopherResponse::Default(_response) => None,
      GopherResponse::Menu(response) => {
        let link = &response.lines[index];
        Some(link.get_url())
      }
    }
  }
}

fn main() {
  println!("Welcome to rs-gopher-client!");

  let mut url = GopherURL::new();
  let mut response: GopherResponse = GopherResponse::Default(GopherDefaultResponse::new());
  loop {
    if let Some(full_url) = url.get_url() {
      println!("\nCurrent page: {}", full_url);
    }
    println!("Please enter command:");

    let mut command = String::new();
    stdin().read_line(&mut command)
      .expect("Failed to read line");

    command = String::from(command.trim());

    if command.starts_with("get ") {
      match command.get(3..) {
        Some(content) => {
          url = GopherURL::from(&content);
        },
        None => {
          println!("Was expecting get [url]");
        },
      }
    }
    else if command.starts_with("f ") {
      match command.get(2..) {
        Some(content) => {
          match &response.get_link_url(&content) {
            Some(link_url) => {
              url = GopherURL::from(&link_url);
            },
            None => {
              println!("Seems there is no link in the current document");
              continue;
            },
          }
        },
        None => {
          println!("Was expecting f [id]");
        },
      }
    }
    else if command.starts_with("quit") {
      println!("Terminated.");
      break;
    }
    else {
      println!("Please enter one of following command:\n\
                \tget [url]: Get this url\n\
                \tf [index]: Follow link index\n\
                \tquit: Quit this program");
      continue;
    }

    if let Some(full_url) = url.get_url() {
      println!("\nGetting {}...", full_url);
    }
    else {
      break;
    }
    match TcpStream::connect(url.get_server()) {
      Ok(mut stream) => {
        println!("Connected!\n");

        stream.write(format!("{}\r\n", url.selector).as_bytes()).unwrap();

        let mut buffer = String::new();

        match stream.read_to_string(&mut buffer) {
          Ok(_) => {
            // Parse Gopher menu according to Gopher selector
            if url.r#type == "1" {
              response = GopherResponse::Menu(GopherMenuResponse::from(&buffer));
            }
            else {
              response = GopherResponse::Default(GopherDefaultResponse::from(&buffer));
            }
            response.display();
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
