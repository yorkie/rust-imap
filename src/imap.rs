
#![crate_id = "imap#0.0.1"]
#![crate_type = "lib"]
#![feature(struct_variant)]
#![feature(phase)]

extern crate openssl;
extern crate collections;
extern crate core;
extern crate regex;

#[phase(pluge)] extern crate regex_macros;

use std::int;
use std::str;

// use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
// use std::io::IoResult;
// use openssl::ssl::{SslStream, SslContext, Sslv23};
use collections::string::String;
use regex::Regex;

// pub enum NetworkStream {
//   NormalStream(TcpStream),
//   SslProtectedStream(SslStream<TcpStream>)
// }

pub enum IMAPCommand {
  Greeting,
  Login,
  Logout,
  Authenticate,
  Select,
  Fetch,
}

pub struct IMAPStream {
  pub host: &'static str,
  pub port: u16,
  pub connected: bool,
  pub authenticated: bool,
  pub selected: bool,
  tag: uint,
  socket: Option<TcpStream>,
  last_command: IMAPCommand
}

fn noop() {
  // nothing
}

impl IMAPStream {
  
  #[inline]
  pub fn new(host: &'static str, port: u16) -> IMAPStream {
    IMAPStream {
      host: host,
      port: port,
      socket: None,
      connected: false,
      selected: false,
      authenticated: false,
      tag: 1,
      last_command: Greeting,
    }
  }
  
  // build connection to host/port(IMAPServer)
  pub fn connect(&mut self) {
    match TcpStream::connect(self.host, self.port) {
      Ok(stream) => {
        self.connected = true;
        self.socket = Some(stream.clone());
        match read_response(self.socket.get_mut_ref(), self.last_command) {
          Ok(_) => return,
          Err(e) => fail!("failed connected, {}", e),
        }
      },
      Err(e) => println!("failed to connect, {}", e),
    }
  }

  // login via username and password
  pub fn login(&mut self, username: &str, password: &str) {
    if !self.connected {
      fail!("connect() required");
    }

    write!(self.socket.get_mut_ref(),
      "x{} login {} {}\r\n", self.tag, username, password);
    self.tag += 1;
    self.last_command = Login;
    match read_response(self.socket.get_mut_ref(), self.last_command) {
      Ok(res) => {
        match res.result.unwrap() {
          IMAPOk => self.authenticated = true,
          IMAPNo => self.authenticated = false,
          IMAPBad => self.authenticated = false,
          _ => fail!("error")
        }
      },
      Err(e) => println!("error: {}", e)
    }
  }

  // authenticate via username and accessToken
  // pub fn auth(&mut self, username: &str, token: &str) {
  //   if !self.connected {
  //     fail!("connect() required");
  //   }
  //   // TODO(Yorkie)
  // }

  // select folder
  pub fn select(&mut self, folder: &str) {
    if !self.authenticated {
      fail!("login()/auth() required");
    }

    write!(self.socket.get_mut_ref(),
      "x{} select {}\r\n", self.tag, folder);
    self.tag += 1;
    self.last_command = Select;
    match read_response(self.socket.get_mut_ref(), self.last_command) {
      Ok(res) => {
        match res.result.unwrap() {
          IMAPFolder {
            exists: exists,
            recent: recent,
            uidvaildity: uidvaildity,
            uidnext: uidnext } => {
            self.selected = true;
            noop();
          },
          _ => fail!("error"),
        }
      },
      Err(e) => println!("error: {}", e)
    }
  }

  // examine
  pub fn examine(&mut self, folder: &str) {
    if !self.authenticated {
      fail!("login()/auth() required");
    }

    write!(self.socket.get_mut_ref(),
      "x{} examine {}\r\n", self.tag, folder);
    self.tag += 1;
    self.last_command = Select;
    match read_response(self.socket.get_mut_ref(), self.last_command) {
      Ok(res) => {
        match res.result.unwrap() {
          IMAPFolder {
            exists: exists,
            recent: recent,
            uidvaildity: uidvaildity,
            uidnext: uidnext } => {
            self.selected = true;
            noop();
          },
          _ => fail!("error"),
        }
      },
      Err(e) => println!("error: {}", e)
    }
  }

  // fetch
  pub fn fetch_by_uid(&mut self, range: (int, int), fields_str: &str) {
    if !self.authenticated {
      fail!("login/auth required");
    }
    if !self.selected {
      fail!("select/examine required");
    }

    write!(self.socket.get_mut_ref(),
      "x{} uid fetch {}\r\n", self.tag, fields_str);
    self.tag += 1;
    self.last_command = Fetch;
    match read_response(self.socket.get_mut_ref(), self.last_command) {
      Ok(res) => {
        match res.result.unwrap() {
          IMAPMessage {..} => {
            noop();
          },
          _ => fail!("error")
        }
      },
      Err(e) => println!("error: {}", e)
    }
  }

  // logout
  pub fn logout(&mut self) {
    if !self.authenticated {
      fail!("connect() required");
    }
    write!(self.socket.get_mut_ref(),
      "x{} logout\r\n", self.tag);
    self.tag = 1;
    self.last_command = Logout;
  }

}

//
// Parsing Response
//

pub enum IMAPResult {
  IMAPOk,
  IMAPNo,
  IMAPBad,
  IMAPFolder {
    recent: int,
    exists: int,
    uidvaildity: int,
    uidnext: int,
  },
  IMAPMessage {
    flags: Vec<String>,
    size: int,
    internal_date: String,
    envelop: Envelop
  }
}

struct Folder {
  exists: int,
  recent: int,
  uidvaildity: int,
  uidnext: int,
}

struct Message {
  flags: Vec<String>,
  size: int,
  internal_date: String,
  envelop: Envelop
}

struct Envelop {
  title: String,
  date: String,
  from: Vec<String>,
  to: Vec<String>,
  cc: Vec<String>,
}

struct IMAPResponse {
  buffer: String,
  lines: Vec<IMAPLine>,
  completed: bool,
  result: Option<IMAPResult>
}

impl IMAPResponse {

  #[inline]
  fn new() -> IMAPResponse {
    IMAPResponse {
      buffer: String::new(),
      lines: Vec::new(),
      completed: false,
      result: None
    }
  }

  #[inline]
  fn add_line(&mut self, mut line: IMAPLine) {
    let line_raw_u8 = line.raw.clone();
    self.completed = line.is_complete();
    self.lines.push(line);
    self.buffer.push_str(
      str::from_utf8(line_raw_u8.as_bytes()).unwrap());

    if self.completed {
      self.parse();
    }
  }

  #[inline]
  fn parse(&mut self) {
    match self.lines.as_slice()[0].command {
      Greeting => self.parse_greeting(),
      Login => self.parse_login(),
      Select => self.parse_select(),
      Fetch => self.parse_fetch(),
      _ => println!("un impl -ed"),
    }
  }

  #[inline]
  fn parse_greeting(&mut self) {
    self.result = Some(IMAPOk)
  }

  #[inline]
  fn parse_login(&mut self) {
    self.result = Some(IMAPOk)
  }

  fn parse_select(&mut self) {
    let mut res = Folder { exists:0, recent:0, uidvaildity:0, uidnext:0 };
    for line in self.lines.iter() {
      let text = str::from_utf8(line.raw.as_bytes()).unwrap();
      let re1;
      let re2;

      // parse recent/exists
      re1 = match Regex::new("([0-9]+) (EXISTS|RECENT)") {
        // TODO(Yorkie): use regex! replace this.
        Ok(re) => re,
        Err(err) => fail!("{}", err),
      };
      match re1.captures(text) {
        Some(caps) => {
          match caps.at(2) {
            "EXISTS" => res.exists = int::parse_bytes(caps.at(1).as_bytes(), 10).unwrap(),
            "RECENT" => res.recent = int::parse_bytes(caps.at(1).as_bytes(), 10).unwrap(),
            _ => noop()
          }
        },
        None => noop()
      }

      // parse uidvaildity/uidnext
      re2 = match Regex::new("(UIDVALIDITY|UIDNEXT) ([0-9]+)") {
        // TODO(Yorkie): use regex! replace this.
        Ok(re) => re,
        Err(err) => fail!("{}", err),
      };
      match re2.captures(text) {
        Some(caps) => {
          match caps.at(1) {
            "UIDVALIDITY" => res.uidvaildity = int::parse_bytes(caps.at(2).as_bytes(), 10).unwrap(),
            "UIDNEXT"     => res.uidnext = int::parse_bytes(caps.at(2).as_bytes(), 10).unwrap(),
            _ => noop()
          }
        },
        None => noop()
      }
    }
    self.result = Some(IMAPFolder {
      recent: res.recent,
      exists: res.exists,
      uidvaildity: res.uidvaildity,
      uidnext: res.uidnext,
    })
  }

  pub fn parse_fetch(&mut self) {
    for line in self.lines.iter() {
      println!("{}", line.raw);
    }
  }

}

struct IMAPLine {
  command: IMAPCommand,
  tagged: bool,
  raw: String,
}

impl IMAPLine {
  fn new(mut bufs: String, cmd: IMAPCommand) -> IMAPLine {
    let mut line = IMAPLine { command: cmd, tagged: false, raw: bufs.clone() };
    let mut cursor = 0i;
    while bufs.len() > 0 {
      let cur_ch = bufs.shift_char().unwrap();
      if cursor == 0 {
        line.tagged = cur_ch != '*';
      }
      cursor += 1;
    }
    return line;
  }
  fn is_complete(&mut self) -> bool {
    match self.command {
      Greeting => true,
      _ => self.tagged,
    }
  }
}

#[inline]
fn read_response(stream: &mut TcpStream, cmd: IMAPCommand) -> Result<Box<IMAPResponse>, Vec<u8>> {
  let mut response = box IMAPResponse::new();
  let mut bufs: Vec<u8> = Vec::new();
  let mut tryClose = false;

  loop {
    let mut buf = [0];
    stream.read(buf);
    bufs.push(buf[0]);

    // check CLRL firstly, if yes then break
    if tryClose && buf[0] == 0x0a {
      match String::from_utf8(bufs.clone()) {
        Ok(res) => {
          let line = IMAPLine::new(res, cmd);
          response.add_line(line);
          if response.completed {
            return Ok(response);
          } else {
            bufs = Vec::new();
          }
        },
        Err(e) => return Err(e),
      }
    }
    tryClose = buf[0] == 0x0d;
  }
}

#[test]
fn create_new_imap_stream() {
  let mut imapstream = IMAPStream::new("imap.qq.com", 143);
  assert!(imapstream.host == "imap.qq.com");
  assert!(imapstream.port == 143);
  imapstream.connect();
}
