use std::net::{TcpStream};
use std::io::{Read, Write, stdin};

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
      port: String::from("70"),
      r#type: String::from("1"),
      selector: String::new(),
    }
  }

  fn from(url: &str) -> GopherURL {
    let mut parsed_url = url;
    // Remove scheme from URL when included
    if url.starts_with("gopher://") {
      parsed_url = &url[9..];
    }

    // Create GopherURL variable to receive the URL
    let mut parsed_gopher_url = GopherURL::new();
    // Split URL on "/" in three first elements
    let url_elements: Vec<&str> = parsed_url.splitn(3, "/").collect();

    // Get host from URL and port if specified
    // If the URL contains a ":", it means the port is specified
    if url_elements[0].contains(":") {
      let port_idx = url_elements[0].find(":").unwrap();
      parsed_gopher_url.host = url_elements[0][0..port_idx].to_string();
      parsed_gopher_url.port = url_elements[0][port_idx + 1..].to_string();
    }
    else {
      parsed_gopher_url.host = url_elements[0].to_string();
    }

    // Get resource type if specified
    if let Some(elm) = url_elements.get(1) {
      parsed_gopher_url.r#type = elm.to_string();
    }

    // Get selector if specified
    if let Some(elm) = url_elements.get(2) {
      // Concatenate "/" which has been removed by the previous .split()
      parsed_gopher_url.selector = "/".to_owned() + &elm.to_string();
    }
    return parsed_gopher_url
  }

  fn get_server(&self) -> String {
    return format!("{}:{}", &self.host, &self.port);
  }

  fn get_url(&self) -> Option<String> {
    if &self.host == "" {
      return None;
    }
    else {
      return Some(format!("gopher://{}:{}/{}{}", &self.host, &self.port, &self.r#type, &self.selector));
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
          return Some(format!("gopher://{}:{}/{}{}", &self.host, &self.port, "1", &self.selector[..idx]));
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
    let splitted_elements: Vec<&str> = line.split("\t").collect();

    GopherMenuLine {
      r#type: splitted_elements[0][0..1].to_string(),
      description: splitted_elements[0][1..].to_string(),
      selector: splitted_elements[1].to_string(),
      host: splitted_elements[2].to_string(),
      port: splitted_elements[3].to_string()
    }
  }

  fn get_url(&self) -> String {
    if &self.host == "" {
      return String::new();
    }
    else {
      return format!("gopher://{}:{}/{}{}", &self.host, &self.port, &self.r#type, &self.selector);
    }
  }
}

struct GopherMenuResponse {
  lines: Vec<GopherMenuLine>,
  links: Vec<usize>,
}

impl GopherMenuResponse {
  fn from(response: &str) -> GopherMenuResponse {
    let mut lines = Vec::new();
    let mut links = Vec::new();

    for (index, line) in response.split("\r\n").enumerate() {
      // dot indicates end of response
      if line.starts_with(".") || line.trim() == "" {
        break;
      }

      let gopherline = GopherMenuLine::from(line);

      // We detect lines which are links and push them into dedicated vector
      if ["0".to_string(), "1".to_string()].contains(&gopherline.r#type) {
        links.push(index);
      }

      lines.push(gopherline);
    }

    GopherMenuResponse {
      lines,
      links,
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
      if line.starts_with(".") {
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
              // We increase the link index by 1 for a more user-friendly display
              let displayed_index = response.links.iter().position(|&x| x == index).unwrap() + 1;
              println!("{}\t[{}]\t{}", resource_type, displayed_index.to_string(), line.description);
            },
            "1" => {
              let resource_type = "MENU";
              // We increase the link index by 1 for a more user-friendly display
              let displayed_index = response.links.iter().position(|&x| x == index).unwrap() + 1;
              println!("{}\t[{}]\t{}/", resource_type, displayed_index.to_string(), line.description);
            },
            "i" => {
              println!("\t\t{}", line.description);
            },
            _ => {
              let resource_type = "UNKNOWN";
              println!("{}\t\t{}", resource_type, line.description);
            },
          }
        }
      }
    }
  }

  fn get_link_url(&self, link_idx: &str) -> Option<String> {
    // Note: Index given by the user has been increased by 1 for a more user-friendly display
    let idx = link_idx.parse::<usize>();
    match idx {
      Ok(index) => {
        match &self { GopherResponse::Text(_response) => None,
          GopherResponse::Menu(response) => {
            // Check if the given index is out of bounds
            if index == 0 || response.links.len() < index {
              return None;
            }
            let link_pointer:usize = response.links[index - 1];
            let link = &response.lines[link_pointer];
            return Some(link.get_url())
          }
        }
      },
      Err(_error) => {
        // May happen when index is negative
        return None;
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
      url = GopherURL::from(&command[4..]);
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
    else if command == "quit" {
      println!("Goodbye!");
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
      Some("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk".to_string()),
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
      Some("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/phlog".to_string()),
      GopherURL::from("zaibatsu.circumlunar.space/0/~solderpunk/phlog/project-gemini.txt").get_url_parent_selector()
    );
    // Menu parent for a menu resource
    assert_eq!(
      Some("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/phlog").get_url_parent_selector()
    );
    // Root menu parent for a menu resource
    assert_eq!(
      Some("gopher://zaibatsu.circumlunar.space:70/1".to_string()),
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
      "gopher://khzae.net:70/0/rfc1436.txt".to_string(),
      GopherMenuLine::from("0RFC 1436 (gopher protocol)	/rfc1436.txt	khzae.net	70").get_url()
    );
  }
}

#[cfg(test)]
mod tests_gopher_menu_response {
  use super::*;

  #[test]
  fn should_return_right_link() {
    let response = "\
isome test		error.host	1\r\n\
i 		error.host	1\r\n\
1About	/about	khzae.net	70\r\n\
i 		error.host	1\r\n\
1Super Dimension Fortress (SDF)	/	sdf.org	70\r\n\
0RFC 4266 (gopher URI scheme)	/rfc4266.txt	khzae.net	70\r\n\
.";
    let parsed_response = GopherResponse::Menu(GopherMenuResponse::from(response));
    assert_eq!(
      Some("gopher://khzae.net:70/1/about".to_string()),
      parsed_response.get_link_url("1")
    );
    assert_eq!(
      Some("gopher://sdf.org:70/1/".to_string()),
      parsed_response.get_link_url("2")
    );
    assert_eq!(
      Some("gopher://khzae.net:70/0/rfc4266.txt".to_string()),
      parsed_response.get_link_url("3")
    );
  }

  #[test]
  fn should_return_none_when_link_out_of_bounds() {
    let response = "\
isome test		error.host	1\r\n\
i 		error.host	1\r\n\
1About	/about	khzae.net	70\r\n\
i 		error.host	1\r\n\
1Super Dimension Fortress (SDF)	/	sdf.org	70\r\n\
0RFC 4266 (gopher URI scheme)	/rfc4266.txt	khzae.net	70\r\n\
.";
    let parsed_response = GopherResponse::Menu(GopherMenuResponse::from(response));
    assert_eq!(
      None,
      parsed_response.get_link_url("-10")
    );
    assert_eq!(
      None,
      parsed_response.get_link_url("0")
    );
    assert_eq!(
      None,
      parsed_response.get_link_url("4")
    );
    assert_eq!(
      None,
      parsed_response.get_link_url("20")
    );
  }
}
