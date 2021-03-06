// Copyright © Killian Kemps (2019)
//
// Killian Kemps <developer@killiankemps.fr>
//
// This software is a computer program whose purpose is to communicate with
// the Gopher protocol.
//
// This software is governed by the CeCILL license under French law and
// abiding by the rules of distribution of free software.  You can  use,
// modify and/ or redistribute the software under the terms of the CeCILL
// license as circulated by CEA, CNRS and INRIA at the following URL
// "http://www.cecill.info".
//
// As a counterpart to the access to the source code and  rights to copy,
// modify and redistribute granted by the license, users are provided only
// with a limited warranty  and the software's author,  the holder of the
// economic rights,  and the successive licensors  have only  limited
// liability.
//
// In this respect, the user's attention is drawn to the risks associated
// with loading,  using,  modifying and/or developing or reproducing the
// software by the user in light of its specific status of free software,
// that may mean  that it is complicated to manipulate,  and  that  also
// therefore means  that it is reserved for developers  and  experienced
// professionals having in-depth computer knowledge. Users are therefore
// encouraged to load and test the software's suitability as regards their
// requirements in conditions enabling the security of their systems and/or
// data to be ensured and,  more generally, to use and operate it in the
// same conditions as regards security.
//
// The fact that you are presently reading this means that you have had
// knowledge of the CeCILL license and that you accept its terms.

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, stdin, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::Path;

const SOFTWARE_NAME: &str = "marmotte";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
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
    let url_elements: Vec<&str> = parsed_url.splitn(2, "/").collect();

    // Get host from URL and port if specified
    // If the URL contains a ":", it means the port is specified
    if url_elements[0].contains(":") {
      let port_idx = url_elements[0].find(":").unwrap();
      parsed_gopher_url.host = url_elements[0][0..port_idx].to_string();
      parsed_gopher_url.port = url_elements[0][port_idx + 1..].to_string();
    } else {
      parsed_gopher_url.host = url_elements[0].to_string();
    }

    // Get resource type and selector if specified
    if let Some(elm) = url_elements.get(1) {
      parsed_gopher_url.r#type = elm[0..1].to_string();
      parsed_gopher_url.selector = elm[1..].to_string();
    }

    return parsed_gopher_url;
  }

  fn get_server(&self) -> String {
    return format!("{}:{}", &self.host, &self.port);
  }

  fn get_url(&self) -> Option<String> {
    if &self.host == "" {
      return None;
    } else {
      return Some(format!(
        "gopher://{}:{}/{}{}",
        &self.host, &self.port, &self.r#type, &self.selector
      ));
    }
  }

  fn get_url_parent_selector(&self) -> Option<String> {
    if &self.host == "" {
      return None;
    }
    // This means we are at the server root, so no parent.
    else if &self.selector == "" {
      return None;
    } else {
      match self.selector.trim_end_matches('/').rfind("/") {
        Some(idx) => {
          return Some(format!(
            "gopher://{}:{}/{}{}",
            &self.host,
            &self.port,
            "1",
            &self.selector[..idx]
          ));
        }
        None => {
          return Some(format!("gopher://{}:{}", &self.host, &self.port));
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
struct GopherMenuLine {
  r#type: String,
  description: String,
  selector: String,
  host: String,
  port: String,
}

impl GopherMenuLine {
  fn from(line: &str) -> Result<GopherMenuLine, String> {
    let splitted_elements: Vec<&str> = line.split("\t").collect();
    // Can't panick as we should at least have an empty item in the vector
    let first_element = splitted_elements[0];

    let item_type = match first_element.get(0..1) {
      Some(el) => el.to_string(),
      None => return Err(format!("Could not parse item type in: \"{}\"", line)),
    };

    // Note: can't return an error because the description will be at least
    // empty if we could already parse the item type before
    let description = match first_element.get(1..) {
      Some(el) => el.to_string(),
      None => return Err(format!("Could not parse description in: \"{}\"", line)),
    };

    let selector = match splitted_elements.get(1) {
      Some(el) => el.to_string(),
      None => return Err(format!("Could not parse selector in: \"{}\"", line)),
    };

    let host = match splitted_elements.get(2) {
      Some(el) => el.to_string(),
      None => return Err(format!("Could not parse host in: \"{}\"", line)),
    };

    let port = match splitted_elements.get(3) {
      Some(el) => el.to_string(),
      None => return Err(format!("Could not parse port in: \"{}\"", line)),
    };

    Ok(GopherMenuLine {
      r#type: item_type,
      description: description,
      selector: selector,
      host: host,
      port: port,
    })
  }

  fn get_url(&self) -> String {
    if &self.host == "" {
      return String::new();
    } else {
      return format!(
        "gopher://{}:{}/{}{}",
        &self.host, &self.port, &self.r#type, &self.selector
      );
    }
  }
}

#[derive(Clone)]
struct GopherMenuResponse {
  lines: Vec<Result<GopherMenuLine, String>>,
  links: Vec<usize>,
}

impl GopherMenuResponse {
  fn from(response: &str) -> GopherMenuResponse {
    let mut lines = Vec::new();
    let mut links = Vec::new();

    for (index, line) in response.split("\r\n").enumerate() {
      // dot indicates end of response
      if line == "." || line.trim() == "" {
        break;
      }

      let gopherline = GopherMenuLine::from(line);

      // We detect lines which are links and push them into dedicated vector
      if let Ok(gopherline) = &gopherline {
        if ["0".to_string(), "1".to_string()].contains(&gopherline.r#type) {
          links.push(index);
        }
      }

      lines.push(gopherline);
    }

    GopherMenuResponse { lines, links }
  }
}

#[derive(Clone)]
struct GopherTextResponse {
  lines: Vec<String>,
}

impl GopherTextResponse {
  fn new() -> GopherTextResponse {
    GopherTextResponse { lines: Vec::new() }
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

    GopherTextResponse { lines }
  }
}

#[derive(Clone)]
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
      }
      GopherResponse::Menu(response) => {
        for (index, line) in response.lines.iter().enumerate() {
          match line {
            Ok(line) => {
              match &line.r#type[..] {
                "0" => {
                  let resource_type = "TXT";
                  // We increase the link index by 1 for a more user-friendly display
                  let displayed_index =
                    response.links.iter().position(|&x| x == index).unwrap() + 1;
                  println!(
                    "{}\t[{}]\t{}",
                    resource_type,
                    displayed_index.to_string(),
                    line.description
                  );
                }
                "1" => {
                  let resource_type = "MENU";
                  // We increase the link index by 1 for a more user-friendly display
                  let displayed_index =
                    response.links.iter().position(|&x| x == index).unwrap() + 1;
                  println!(
                    "{}\t[{}]\t{}/",
                    resource_type,
                    displayed_index.to_string(),
                    line.description
                  );
                }
                "i" => {
                  println!("\t\t{}", line.description);
                }
                _ => {
                  let resource_type = "UNKNOWN";
                  println!("{}\t\t{}", resource_type, line.description);
                }
              }
            }
            Err(line) => println!("ERR\t\tmarmotte: Problem parsing line {}: {}", index, line),
          }
        }
      }
    }
  }

  fn get_link_url(&self, link_idx: &str) -> Result<String, String> {
    // Note: Index given by the user has been increased by 1 for a more user-friendly display
    let idx = link_idx.parse::<usize>();
    match idx {
      Ok(index) => {
        match &self {
          GopherResponse::Text(_response) => {
            Err("There is no link in the current document".to_string())
          }
          GopherResponse::Menu(response) => {
            // Check if the given index is out of bounds
            if index == 0 || response.links.len() < index {
              return Err("Given index is out of bounds".to_string());
            }
            let link_pointer: usize = response.links[index - 1];
            match &response.lines[link_pointer] {
              Ok(link) => Ok(link.get_url()),
              Err(msg) => Err(format!("Chosen link as an issue: {}", msg)),
            }
          }
        }
      }
      Err(_error) => {
        // May happen when index is negative
        return Err("Link index can't be negative".to_string());
      }
    }
  }
}

fn manage_url_request(url: GopherURL, state: &mut ClientState) {
  match TcpStream::connect(url.get_server()) {
    Ok(mut stream) => {
      stream
        .write(format!("{}\r\n", url.selector).as_bytes())
        .unwrap();

      let mut buffer = String::new();

      match stream.read_to_string(&mut buffer) {
        Ok(_) => {
          // Parse Gopher menu according to Gopher selector
          if url.r#type == "1" {
            state.last_response = GopherResponse::Menu(GopherMenuResponse::from(&buffer));
          } else {
            state.last_response = GopherResponse::Text(GopherTextResponse::from(&buffer));
          }
          state.last_response.display();
          // Insert displayed page to history
          state.history.insert(0, url);
        }
        Err(e) => {
          println!("Failed to receive data: {}", e);
        }
      }
    }
    Err(e) => {
      println!("Failed to connect: {}", e);
    }
  }
}

#[derive(Clone)]
struct ClientState {
  bookmarks: Vec<GopherURL>,
  history: Vec<GopherURL>,
  last_response: GopherResponse,
}

impl ClientState {
  // Get back URL and update history
  // We update the history because we are going back and rewriting it
  // We remove the back URL because it will be put to history again after being requested
  fn prepare_going_back(&mut self) -> Result<GopherURL, String> {
    // Get second-to-last url
    match self.history.get(1) {
      Some(_) => {
        // Remove second-to-last url
        let previous_url = self.history.remove(1);
        // Remove last url
        self.history.remove(0);
        return Ok(previous_url);
      }
      None => {
        return Err("There is no previous document to go back".to_string());
      }
    }
  }

  // Get back URL and send request
  fn go_back(&mut self) -> Result<String, String> {
    match self.prepare_going_back() {
      Ok(previous_url) => {
        // Load previous url
        manage_url_request(previous_url, self);
        return Ok("Went back to previous document".to_string());
      }
      Err(msg) => {
        return Err(msg);
      }
    }
  }

  fn open_bookmarks(&mut self, write: bool) -> Result<std::fs::File, String> {
    // Use HOME variable to locate bookmarks
    match env::var("HOME") {
      Ok(home) => {
        let bookmarks_location = format!("{}/.{}/bookmarks.txt", home, SOFTWARE_NAME);
        let bookmarks_file = if write {
          OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&bookmarks_location)
        } else {
          File::open(&bookmarks_location)
        };

        match bookmarks_file {
          Ok(file) => Ok(file),
          Err(error) => match error.kind() {
            ErrorKind::NotFound => {
              let bookmarks_path = Path::new(&bookmarks_location);
              let prefix = bookmarks_path.parent().unwrap();

              // Create folder for bookmarks and then file
              match std::fs::create_dir_all(prefix) {
                Ok(_) => match File::create(bookmarks_location.to_string()) {
                  Ok(created_file) => Ok(created_file),
                  Err(e) => Err(format!("Problem creating the bookmarks file: {:?}", e)),
                },
                Err(e) => Err(format!(
                  "Problem creating folder to store bookmarks file: {:?}",
                  e
                )),
              }
            }
            _ => Err(format!("Problem reading the bookmarks file: {:?}", error)),
          },
        }
      }
      Err(e) => Err(format!(
        "Could not get path to bookmarks because $HOME is not set: {:?}",
        e
      )),
    }
  }

  fn load_bookmarks(&mut self) {
    let f = self.open_bookmarks(false);
    match f {
      Ok(file) => {
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        match buf_reader.read_to_string(&mut contents) {
          Ok(_) => {
            let mut bookmarks: Vec<GopherURL> = Vec::new();
            for line in contents.trim().split("\n") {
              let url = GopherURL::from(line);
              bookmarks.push(url);
            }
            self.bookmarks = bookmarks;
          }
          // This error may happen if the file has been created and has only
          // write permission.
          Err(_) => {
            self.bookmarks = Vec::new();
          }
        }
      }
      Err(e) => {
        println!("{}", e);
        self.bookmarks = Vec::new();
      }
    }
  }

  fn save_bookmarks(&mut self) {
    let f = self.open_bookmarks(true);
    match f {
      Ok(mut file) => {
        for url in self.bookmarks.iter() {
          writeln!(file, "{}", url.get_url().unwrap()).unwrap();
        }
      }
      Err(e) => {
        println!("{}", e);
        self.bookmarks = Vec::new();
      }
    }
  }

  fn display_bookmarks(&self) {
    if &self.bookmarks.len() > &0 {
      println!("Bookmarks:");
      for (index, link) in self.bookmarks.iter().enumerate() {
        println!("[bk {}] {}", index, link.get_url().unwrap());
      }
    } else {
      println!("\nThere are no bookmarks");
    }
  }
}

#[derive(Debug, PartialEq)]
enum Commands {
  Up,
  Back,
  GoURL(String),
  GoIndex(String),
  DisplayBookmarks,
  AddBookmark(String),
  RemoveBookmark(String),
  GoBookmarkIndex(String),
  Help,
  Quit,
}

impl Commands {
  fn parse(input: String) -> Result<Commands, String> {
    let mut args = "".to_string();
    let mut command = input.trim().to_string();
    if let Some(index) = command.find(" ") {
      args = command.split_off(index).trim().to_string();
    }

    match &command[..] {
      "up" => Ok(Commands::Up),
      "back" => Ok(Commands::Back),
      "quit" => Ok(Commands::Quit),
      "go" => {
        if args == "" {
          return Err("No URL to go to".to_string());
        }
        return Ok(Commands::GoURL(args));
      }
      "bk" | "bookmarks" if args == "" => Ok(Commands::DisplayBookmarks),
      "bk" | "bookmarks" if args.starts_with(char::is_numeric) => {
        return Ok(Commands::GoBookmarkIndex(args));
      }
      "bk" | "bookmarks" => {
        // Parsing again to get subcommands
        let mut command = args.clone();
        if let Some(index) = command.find(" ") {
          args = command.split_off(index).trim().to_string();
        }
        match &command[..] {
          "add" => Ok(Commands::AddBookmark(args)),
          "rm" => Ok(Commands::RemoveBookmark(args)),
          _ => Err("Bookmark subcommand not found".to_string()),
        }
      }
      _ => {
        if command.starts_with(char::is_numeric) {
          return Ok(Commands::GoIndex(command));
        } else {
          return Ok(Commands::Help);
        }
      }
    }
  }

  fn help() {
    println!(
      "Please enter one of the following commands:\n\
       \tgo [url]: Go to this url\n\
       \t[index]: Follow link index\n\
       \tup: Go up one directory\n\
       \tback: Go back previous page\n\
       \tbk: List bookmarks\n\
       \tbk [index]: Follow bookmark\n\
       \tbk add [url]: Add bookmark\n\
       \tbk rm [index]: Remove bookmark\n\
       \tquit: Quit this program"
    );
  }
}

fn main() {
  println!(
    "Welcome to {} v{}!",
    SOFTWARE_NAME.to_string(),
    VERSION.to_string()
  );
  println!(
    "Enter 'help' if you don't know how to start. Have a nice journey in the Gopherspace!\n"
  );

  let mut state = ClientState {
    bookmarks: Vec::new(),
    history: Vec::new(),
    last_response: GopherResponse::Text(GopherTextResponse::new()),
  };
  state.load_bookmarks();

  // Get directly page if URL provided as argument
  let args: Vec<String> = env::args().collect();
  if let Some(url) = args.get(1) {
    let parsed_url = GopherURL::from(url);
    manage_url_request(parsed_url, &mut state);
  }

  loop {
    if let Some(last_url) = state.history.get(0) {
      if let Some(full_url) = last_url.get_url() {
        println!("\nCurrent page: {}", full_url);
      }
    }
    print!("{}> ", SOFTWARE_NAME.to_string());
    io::stdout().flush().unwrap();

    let mut command_input = String::new();
    stdin()
      .read_line(&mut command_input)
      .expect("Failed to read line");

    let command = Commands::parse(command_input);

    match command {
      Ok(Commands::GoURL(url)) => {
        let gopher_url = GopherURL::from(&url);
        manage_url_request(gopher_url, &mut state);
      }
      Ok(Commands::GoIndex(index)) => match &state.last_response.get_link_url(&index) {
        Ok(link_url) => {
          let url = GopherURL::from(&link_url);
          manage_url_request(url, &mut state);
        }
        Err(msg) => {
          println!("{}", msg);
          continue;
        }
      },
      Ok(Commands::Up) => match state.history.get(0) {
        Some(last_url) => match last_url.get_url_parent_selector() {
          Some(parent_url) => {
            let url = GopherURL::from(&parent_url);
            manage_url_request(url, &mut state);
          }
          None => {
            println!("Seems there is no parent for this document");
            continue;
          }
        },
        None => {
          println!("There is no current document");
          continue;
        }
      },
      Ok(Commands::Back) => match state.go_back() {
        Ok(_msg) => {}
        Err(msg) => {
          println!("{}", msg);
          continue;
        }
      },
      Ok(Commands::DisplayBookmarks) => state.display_bookmarks(),
      Ok(Commands::GoBookmarkIndex(args)) => {
        let index = match args.parse::<usize>() {
          Ok(i) => i,
          Err(error) => {
            println!("Could not parse the bookmarks index: {:?}", error);
            continue;
          }
        };
        if let Some(url) = state.bookmarks.get(index) {
          // We need to url.clone() because the URL needs to be kept in the
          // bookmarks AND in the browsing history
          manage_url_request(url.clone(), &mut state);
        } else {
          println!("There is no bookmark at this index");
        }
      }
      Ok(Commands::AddBookmark(args)) => {
        let url = GopherURL::from(&args);
        state.bookmarks.push(url);
        state.save_bookmarks();
        state.display_bookmarks();
      }
      Ok(Commands::RemoveBookmark(args)) => {
        let index = match args.parse::<usize>() {
          Ok(i) => i,
          Err(error) => {
            println!("Could not parse the bookmarks index: {:?}", error);
            continue;
          }
        };
        state.bookmarks.remove(index);
        state.save_bookmarks();
        state.display_bookmarks();
      }
      Err(msg) => println!("Command parsing error: {}", msg),
      Ok(Commands::Quit) => {
        println!("Goodbye!");
        break;
      }
      _ => {
        Commands::help();
        continue;
      }
    }
  }
}

#[cfg(test)]
mod tests_gopher_url {
  use super::*;

  impl PartialEq for GopherURL {
    fn eq(&self, other: &Self) -> bool {
      self.host == other.host
        && self.port == other.port
        && self.r#type == other.r#type
        && self.selector == other.selector
    }
  }

  #[test]
  fn should_import_any_valid_url() {
    let mut expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "/~solderpunk/".to_string(),
    };
    // Complete Gopher URL
    assert_eq!(
      expected,
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/")
    );
    // Without gopher://
    assert_eq!(
      expected,
      GopherURL::from("zaibatsu.circumlunar.space:70/1/~solderpunk/")
    );
    // With gopher:// but without port number
    assert_eq!(
      expected,
      GopherURL::from("gopher://zaibatsu.circumlunar.space/1/~solderpunk/")
    );
    // Without gopher:// and without port number
    assert_eq!(
      expected,
      GopherURL::from("zaibatsu.circumlunar.space/1/~solderpunk/")
    );

    expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "".to_string(),
    };
    // Hostname only
    assert_eq!(expected, GopherURL::from("zaibatsu.circumlunar.space"));

    expected = GopherURL {
      host: "zaibatsu.circumlunar.space".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "/~solderpunk/phlog/project-gemini.txt".to_string(),
    };
    // Text resource URL
    assert_eq!(
      expected,
      GopherURL::from("zaibatsu.circumlunar.space/0/~solderpunk/phlog/project-gemini.txt")
    );

    expected = GopherURL {
      host: "khzae.net".to_string(),
      port: "105".to_string(),
      r#type: "1".to_string(),
      selector: "/".to_string(),
    };
    // Non-standard port
    assert_eq!(expected, GopherURL::from("khzae.net:105/1/"));

    expected = GopherURL {
      host: "alexschroeder.ch".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "Alex_Schroeder".to_string(),
    };
    // Selector without '/'
    assert_eq!(
      expected,
      GopherURL::from("gopher://alexschroeder.ch/0Alex_Schroeder")
    );
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
      GopherURL::from("zaibatsu.circumlunar.space/0/~solderpunk/phlog/project-gemini.txt")
        .get_url_parent_selector()
    );
    // Menu parent for a text resource without '/'
    assert_eq!(
      Some("gopher://alexschroeder.ch:70".to_string()),
      GopherURL::from("gopher://alexschroeder.ch:70/0Alex_Schroeder").get_url_parent_selector()
    );
    // Menu parent for a menu resource
    assert_eq!(
      Some("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/phlog")
        .get_url_parent_selector()
    );
    // Root menu parent for a menu resource
    assert_eq!(
      Some("gopher://zaibatsu.circumlunar.space:70/1".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk")
        .get_url_parent_selector()
    );
    // Root menu parent for a menu resource
    assert_eq!(
      Some("gopher://zaibatsu.circumlunar.space:70/1".to_string()),
      GopherURL::from("gopher://zaibatsu.circumlunar.space:70/1/~solderpunk/")
        .get_url_parent_selector()
    );
  }
}

#[cfg(test)]
mod tests_gopher_menu_line {
  use super::*;

  impl PartialEq for GopherMenuLine {
    fn eq(&self, other: &Self) -> bool {
      self.host == other.host
        && self.port == other.port
        && self.r#type == other.r#type
        && self.selector == other.selector
        && self.description == other.description
    }
  }

  #[test]
  fn should_import_any_menu_line() {
    let mut expected = GopherMenuLine {
      host: "gopher.floodgap.com".to_string(),
      port: "70".to_string(),
      r#type: "1".to_string(),
      selector: "/home".to_string(),
      description: "Floodgap Home".to_string(),
    };
    // Menu line
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("1Floodgap Home	/home	gopher.floodgap.com	70")
    );

    expected = GopherMenuLine {
      host: "error.host".to_string(),
      port: "1".to_string(),
      r#type: "i".to_string(),
      selector: "".to_string(),
      description: "              ,-.      .-,".to_string(),
    };
    // Information line with graphics
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("i              ,-.      .-,		error.host	1")
    );

    expected = GopherMenuLine {
      host: "error.host".to_string(),
      port: "1".to_string(),
      r#type: "i".to_string(),
      selector: "".to_string(),
      description: "Find movie showtimes by postal code/zip.".to_string(),
    };
    // Information line with text
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("iFind movie showtimes by postal code/zip.		error.host	1")
    );

    expected = GopherMenuLine {
      host: "khzae.net".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "/rfc1436.txt".to_string(),
      description: "RFC 1436 (gopher protocol)".to_string(),
    };
    // Text resource line
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("0RFC 1436 (gopher protocol)	/rfc1436.txt	khzae.net	70")
    );

    expected = GopherMenuLine {
      host: "khzae.net".to_string(),
      port: "70".to_string(),
      r#type: "7".to_string(),
      selector: "/dict/search".to_string(),
      description: "Search dictionary".to_string(),
    };
    // Search resource line
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("7Search dictionary	/dict/search	khzae.net	70")
    );

    expected = GopherMenuLine {
      host: "host2".to_string(),
      port: "70".to_string(),
      r#type: "0".to_string(),
      selector: "moo selector".to_string(),
      description: "Some file or other".to_string(),
    };
    // Gopher+ Text resource line
    assert_eq!(
      Ok(expected),
      GopherMenuLine::from("0Some file or other	moo selector	host2	70	+")
    );
  }

  #[test]
  fn should_return_formatted_attributes() {
    // get_url()
    assert_eq!(
      "gopher://khzae.net:70/0/rfc1436.txt".to_string(),
      GopherMenuLine::from("0RFC 1436 (gopher protocol)	/rfc1436.txt	khzae.net	70")
        .unwrap()
        .get_url()
    );
  }

  #[test]
  fn should_manage_parsing_errors() {
    assert_eq!(
      Err("Could not parse item type in: \"\t\t\'\'.                  ....                            \t70\"".to_string()),
      GopherMenuLine::from("		''.                  ....                            	70")
    );

    assert_eq!(
      Err("Could not parse selector in: \"idescription   \"".to_string()),
      GopherMenuLine::from("idescription   ")
    );

    assert_eq!(
      Err("Could not parse host in: \"idescription\tselector\"".to_string()),
      GopherMenuLine::from("idescription	selector")
    );

    assert_eq!(
      Err("Could not parse port in: \"ior taken the time to contribute in other way. false\tnull.host\t1\"".to_string()),
      GopherMenuLine::from("ior taken the time to contribute in other way. false	null.host	1")
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
      Ok("gopher://khzae.net:70/1/about".to_string()),
      parsed_response.get_link_url("1")
    );
    assert_eq!(
      Ok("gopher://sdf.org:70/1/".to_string()),
      parsed_response.get_link_url("2")
    );
    assert_eq!(
      Ok("gopher://khzae.net:70/0/rfc4266.txt".to_string()),
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
      Err("Link index can\'t be negative".to_string()),
      parsed_response.get_link_url("-10")
    );
    assert_eq!(
      Err("Given index is out of bounds".to_string()),
      parsed_response.get_link_url("0")
    );
    assert_eq!(
      Err("Given index is out of bounds".to_string()),
      parsed_response.get_link_url("4")
    );
    assert_eq!(
      Err("Given index is out of bounds".to_string()),
      parsed_response.get_link_url("20")
    );
  }
}

#[cfg(test)]
mod tests_state {
  use super::*;

  #[test]
  fn should_prepare_going_back() {
    // Set initial state
    let current_page = GopherURL::from("gopher://khzae.net");
    let mut state = ClientState {
      bookmarks: Vec::new(),
      history: Vec::new(),
      last_response: GopherResponse::Text(GopherTextResponse::new()),
    };
    state.history.insert(0, current_page);
    state.history.insert(
      1,
      GopherURL::from("gopher://zaibatsu.circumlunar.space/1/~solderpunk"),
    );
    state
      .history
      .insert(2, GopherURL::from("gopher://zaibatsu.circumlunar.space"));

    // Set expected state
    let expected_last_page_history = GopherURL::from("gopher://zaibatsu.circumlunar.space");
    let expected_previous_url =
      GopherURL::from("gopher://zaibatsu.circumlunar.space/1/~solderpunk");
    let mut expected_state = ClientState {
      bookmarks: Vec::new(),
      history: Vec::new(),
      last_response: GopherResponse::Text(GopherTextResponse::new()),
    };
    expected_state.history.push(expected_last_page_history);

    // Get back url
    let previous_url = state.prepare_going_back();

    assert_eq!(expected_state.history, state.history);
    assert_eq!(Ok(expected_previous_url), previous_url);
  }
}

#[cfg(test)]
mod tests_commands {
  use super::*;

  #[test]
  fn should_parse_valid_commands() {
    assert_eq!(Ok(Commands::Up), Commands::parse("up".to_string()));
    assert_eq!(Ok(Commands::Back), Commands::parse("back".to_string()));
    assert_eq!(Ok(Commands::Quit), Commands::parse("quit".to_string()));
    assert_eq!(
      Ok(Commands::GoURL("gopherpedia.com".to_string())),
      Commands::parse("go gopherpedia.com".to_string())
    );
    assert_eq!(
      Ok(Commands::DisplayBookmarks),
      Commands::parse("bk".to_string())
    );
    assert_eq!(
      Ok(Commands::AddBookmark("gopherpedia.com".to_string())),
      Commands::parse("bk add gopherpedia.com".to_string())
    );
    assert_eq!(
      Ok(Commands::RemoveBookmark("2".to_string())),
      Commands::parse("bk rm 2".to_string())
    );
    assert_eq!(
      Ok(Commands::GoBookmarkIndex("2".to_string())),
      Commands::parse("bk 2".to_string())
    );
    assert_eq!(
      Ok(Commands::GoIndex("2".to_string())),
      Commands::parse("2".to_string())
    );
    assert_eq!(Ok(Commands::Help), Commands::parse("help".to_string()));
  }

  #[test]
  fn should_parse_commands_with_spaces() {
    assert_eq!(Ok(Commands::Up), Commands::parse("  up  ".to_string()));
    assert_eq!(Ok(Commands::Back), Commands::parse(" back ".to_string()));
    assert_eq!(Ok(Commands::Quit), Commands::parse("  quit   ".to_string()));
    assert_eq!(
      Ok(Commands::GoURL("gopherpedia.com".to_string())),
      Commands::parse("go   gopherpedia.com".to_string())
    );
    assert_eq!(
      Ok(Commands::DisplayBookmarks),
      Commands::parse("bk".to_string())
    );
    assert_eq!(
      Ok(Commands::AddBookmark("gopherpedia.com".to_string())),
      Commands::parse("bk   add   gopherpedia.com".to_string())
    );
    assert_eq!(
      Ok(Commands::RemoveBookmark("2".to_string())),
      Commands::parse("bk   rm   2  ".to_string())
    );
    assert_eq!(
      Ok(Commands::GoBookmarkIndex("2".to_string())),
      Commands::parse("bk   2 ".to_string())
    );
    assert_eq!(
      Ok(Commands::GoIndex("2".to_string())),
      Commands::parse("  2 ".to_string())
    );
    assert_eq!(Ok(Commands::Help), Commands::parse("help  ".to_string()));
  }

  #[test]
  fn should_ignore_invalid_commands() {
    assert_eq!(
      Ok(Commands::Help),
      Commands::parse("fly me to the moon".to_string())
    );
    assert_eq!(
      Err("Bookmark subcommand not found".to_string()),
      Commands::parse("bk something".to_string())
    );
  }
}
