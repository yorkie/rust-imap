
#![crate_id = "imap#0.0.1"]
#![crate_type = "lib"]

extern crate openssl;
extern crate collections;
extern crate core;

use std::str;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use openssl::ssl::{SslStream, SslContext, Sslv23};
use collections::string::String;
use core::char;

pub enum NetworkStream {
  NormalStream(TcpStream),
  SslProtectedStream(SslStream<TcpStream>)
}

pub struct IMAPStream {
  pub host: &'static str,
  pub port: u16,
  socket: Option<TcpStream>,
  pub connected: bool,
  pub authenticated: bool,
  tag: uint,
}

impl IMAPStream {
  
  #[inline]
  pub fn new(host: &'static str, port: u16) -> IMAPStream {
    IMAPStream {
      host: host,
      port: port,
      socket: None,
      connected: false,
      authenticated: false,
      tag: 1
    }
  }
  
  // build connection to host/port(IMAPServer)
  pub fn connect(&mut self) {
    match TcpStream::connect(self.host, self.port) {
      Ok(stream) => {
        self.connected = true;
        self.socket = Some(stream.clone());
        match read_response(self.socket.get_mut_ref()) {
          Ok(res) => return,
          Err(e) => drop(stream),
        }
      },
      Err(e) => println!("Failed to connect"),
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
    match read_response(self.socket.get_mut_ref()) {
      Ok(res) => {
        self.authenticated = true;
        println!("response: {}", res);
      },
      Err(e) => println!("error"),
    }
  }

  // authenticate via username and accessToken
  pub fn auth(&mut self, username: &str, token: &str) {
    if !self.connected {
      fail!("connect() required");
    }
    // TODO(Yorkie)
  }

  // select folder
  pub fn select(&mut self, folder: &str) {
    if !self.authenticated {
      fail!("login()/auth() required");
    }

    write!(self.socket.get_mut_ref(),
      "x{} select {}\r\n", self.tag, folder);
    self.tag += 1;
    match read_response(self.socket.get_mut_ref()) {
      Ok(res) => {
        println!("response: {}", res);
      },
      Err(e) => println!("error"),
    }
  }

}

//
// Parsing Response
//

struct IMAPResponse {
  buffer: String,
  lines: Vec<IMAPLine>,
  tagged: bool,
}

impl IMAPResponse {

  #[inline]
  fn new() -> IMAPResponse {
    IMAPResponse {
      buffer: String::new(),
      lines: Vec::new(),
      tagged: false,
    }
  }

  #[inline]
  fn add_line(&mut self, line: IMAPLine) {
    let mut line_raw_u8 = line.raw.clone();
    self.lines.push(line);
    self.buffer.push_str(
      str::from_utf8(line_raw_u8.as_bytes()).unwrap());
  }

}

struct IMAPLine {
  tagged: bool,
  raw: Box<String>,
}

impl IMAPLine {
  fn new(bufs: String) -> IMAPLine {
    let mut line = IMAPLine { tagged: false, raw: box bufs };
    let mut cursor = 0i;
    while line.raw.len() > 0 {
      if cursor == 0 {
        line.tagged = line.raw.shift_char().unwrap() != '*';
      }
      cursor += 1;
    }
    return line;
  }
}

#[inline]
fn read_response(stream: &mut TcpStream) -> Result<String, Vec<u8>> {
  let mut response = box IMAPResponse::new();
  let mut bufs: Vec<u8> = Vec::new();
  let mut tryClose = false;
  loop {
    let mut buf = [0];
    stream.read(buf);
    bufs.push(buf[0]);

    // check CLRL firstly, if yes then break
    if tryClose && buf[0] == 0x0a {
      match String::from_utf8(bufs) {
        Ok(res) => response.add_line(IMAPLine::new(res)),
        Err(e) => return Err(e),
      }
      break;
    }
    tryClose = buf[0] == 0x0d;
  }
  Ok(String::new())
}

#[test]
fn create_new_imap_stream() {
  let mut imapstream = IMAPStream::new("imap.qq.com", 143);
  assert!(imapstream.host == "imap.qq.com");
  assert!(imapstream.port == 143);
  imapstream.connect();
}
