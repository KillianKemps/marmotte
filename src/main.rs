use std::net::{TcpStream};
use std::io::{Read, Write, stdin};
use regex::Regex;

#[derive(Debug)]
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
      selector: captures.get(5).map_or("", |m| m.as_str()).to_string(),
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

  fn get_url_parent_selector(&self) -> Option<String> {
    if &self.host == "" {
      return None;
    }
    // This means we are at the server root, so no parent.
    else if &self.selector == "" {
      return None;
    }
    else {
      match self.selector.rfind("/") {
        Some(idx) => {
          return Some(format!("{}:{}/{}{}", &self.host, &self.port, "1", &self.selector[..idx]));
        },
        None => None,
      }
    }
  }
}

#[derive(Debug)]
struct GopherMenuLine {
  r#type: String,
  description: String,
  selector: String,
  host: String,
  port: String,
}

impl GopherMenuLine {
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
      if line == "." || line.trim() == "" {
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

struct GopherTextResponse {
  lines: Vec<String>,
}

impl GopherTextResponse {
  fn new() -> GopherTextResponse {
    GopherTextResponse {
      lines: Vec::new(),
    }
  }

  fn from(response: &str) -> GopherTextResponse {
    let mut lines = Vec::new();

    for line in response.split("\n").collect::<Vec<&str>>() {
      // dot indicates end of response
      if line == "." {
        break;
      }

      lines.push(line.to_string());
    }

    GopherTextResponse {
      lines,
    }
  }
}

enum GopherResponse {
  Text(GopherTextResponse),
  Menu(GopherMenuResponse),
}

impl GopherResponse {
  fn display(&self) {
    match &self {
      GopherResponse::Text(response) => {
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
      GopherResponse::Text(_response) => None,
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
  let mut response: GopherResponse = GopherResponse::Text(GopherTextResponse::new());
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
    else if command.starts_with(char::is_numeric) {
      match &response.get_link_url(&command) {
        Some(link_url) => {
          url = GopherURL::from(&link_url);
        },
        None => {
          println!("Seems there is no link in the current document");
          continue;
        },
      }
    }
    else if command == "up" {
      match url.get_url_parent_selector() {
        Some(parent_url) => {
          url = GopherURL::from(&parent_url);
        },
        None => {
          println!("Seems there is no parent for this document");
          continue;
        },
      }
    }
    else if command.starts_with("quit") {
      println!("Terminated.");
      break;
    }
    else {
      println!("Please enter one of the following commands:\n\
                \tget [url]: Get this url\n\
                \t[index]: Follow link index\n\
                \tup: Go up one directory\n\
                \tquit: Quit this program");
      continue;
    }

    if let Some(full_url) = url.get_url() {
      println!("\nGetting {}...\r", full_url);
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
              response = GopherResponse::Text(GopherTextResponse::from(&buffer));
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

#[cfg(test)]
mod tests_gopher_url {
  use super::*;

  impl PartialEq for GopherURL {
    fn eq(&self, other: &Self) -> bool {
      self.host == other.host &&
      self.port == other.port &&
      self.r#type == other.r#type &&
      self.selector == other.selector
    }
  }

  #[test]
  fn should_import_any_valid_url() {
    let mut expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "/~solderpunk/".to_string()
    };
    // Complete Gopher URL
    assert_eq!(expected, GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/"));
    // Without gopher://
    assert_eq!(expected, GopherURL::from("zaibatsu.circumlunar.space:70/1/~solderpunk/"));
    // With gopher:// but without port number
    assert_eq!(expected, GopherURL::from("gopher://zaibatsu.circumlunar.space/1/~solderpunk/"));
    // Without gopher:// and without port number
    assert_eq!(expected, GopherURL::from("zaibatsu.circumlunar.space/1/~solderpunk/"));

    expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "".to_string()
    };
    // Hostname only
    assert_eq!(expected, GopherURL::from("zaibatsu.circumlunar.space"));

    expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "/~solderpunk/phlog/project-gemini.txt".to_string()
    };
    // Text resource URL
    assert_eq!(expected, GopherURL::from("zaibatsu.circumlunar.space/0/~solderpunk/phlog/project-gemini.txt"));

    expected = GopherURL {
      host: "khzae.net".to_string(),
      port: "105".to_string(),
      r#type: "1".to_string(),
      selector: "/".to_string()
    };
    // Non-standard port
    assert_eq!(expected, GopherURL::from("khzae.net:105/1/"));
  }

  #[test]
  fn should_return_formatted_attributes() {
    // get_server()
    assert_eq!(
      "zaibatsu.circumlunar.space:70".to_string(),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/").get_server()
    );
    // get_url()
    assert_eq!(
      Some("zaibatsu.circumlunar.space:70/1/~solderpunk".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk").get_url()
    );
  }

  #[test]
  fn should_return_parent_selector_option() {
    // None when already at root even with resource type
    assert_eq!(
      None,
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1").get_url_parent_selector()
    );
    // None when already at root
    assert_eq!(
      None,
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70").get_url_parent_selector()
    );
    // Menu parent for a text resource
    assert_eq!(
      Some("zaibatsu.circumlunar.space:70/1/~solderpunk/phlog".to_string()),
      GopherURL::from("zaibatsu.circumlunar.space/0/~solderpunk/phlog/project-gemini.txt").get_url_parent_selector()
    );
    // Menu parent for a menu resource
    assert_eq!(
      Some("zaibatsu.circumlunar.space:70/1/~solderpunk".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/phlog").get_url_parent_selector()
    );
    // Root menu parent for a menu resource
    assert_eq!(
      Some("zaibatsu.circumlunar.space:70/1".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk").get_url_parent_selector()
    );
  }
}

#[cfg(test)]
mod tests_gopher_menu_line {
  use super::*;

  impl PartialEq for GopherMenuLine {
    fn eq(&self, other: &Self) -> bool {
      self.host == other.host &&
      self.port == other.port &&
      self.r#type == other.r#type &&
      self.selector == other.selector &&
      self.description == other.description
    }
  }

  #[test]
  fn should_import_any_menu_line() {
    let mut expected = GopherMenuLine {
      host: "gopher.floodgap.com".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "/home".to_string(),
      description: "Floodgap Home".to_string()
    };
    // Menu line
    assert_eq!(expected, GopherMenuLine::from("1Floodgap Home	/home	gopher.floodgap.com	70"));

    expected = GopherMenuLine {
      host: "error.host".to_string(),
      port: "1".to_string(),
      r#type: "i".to_string(),
      selector: "".to_string(),
      description: "              ,-.      .-,".to_string()
    };
    // Information line with graphics
    assert_eq!(expected, GopherMenuLine::from("i              ,-.      .-,		error.host	1"));

    expected = GopherMenuLine {
      host: "error.host".to_string(),
      port: "1".to_string(),
      r#type: "i".to_string(),
      selector: "".to_string(),
      description: "Find movie showtimes by postal code/zip.".to_string()
    };
    // Information line with text
    assert_eq!(expected, GopherMenuLine::from("iFind movie showtimes by postal code/zip.		error.host	1"));

    expected = GopherMenuLine {
      host: "khzae.net".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "/rfc1436.txt".to_string(),
      description: "RFC 1436 (gopher protocol)".to_string()
    };
    // Text resource line
    assert_eq!(expected, GopherMenuLine::from("0RFC 1436 (gopher protocol)	/rfc1436.txt	khzae.net	70"));

    expected = GopherMenuLine {
      host: "khzae.net".to_string(),
      port: "70".to_string(),
      r#type: "7".to_string(),
      selector: "/dict/search".to_string(),
      description: "Search dictionary".to_string()
    };
    // Search resource line
    assert_eq!(expected, GopherMenuLine::from("7Search dictionary	/dict/search	khzae.net	70"));

    expected = GopherMenuLine {
      host: "host2".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "moo selector".to_string(),
      description: "Some file or other".to_string()
    };
    // Gopher+ Text resource line
    assert_eq!(expected, GopherMenuLine::from("0Some file or other	moo selector	host2	70	+"));
  }

  #[test]
  fn should_return_formatted_attributes() {
    // get_url()
    assert_eq!(
      "khzae.net:70/0/rfc1436.txt".to_string(),
      GopherMenuLine::from("0RFC 1436 (gopher protocol)	/rfc1436.txt	khzae.net	70").get_url()
    );
  }
}
