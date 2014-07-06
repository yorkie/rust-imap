
#![crate_id = "imap#0.0.1"]
#![crate_type = "lib"]

extern crate openssl;
extern crate collections;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use openssl::ssl::{SslStream, SslContext, Sslv23};
use collections::string::String;

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
}

impl IMAPStream {
  
  #[inline]
  pub fn new(host: &'static str, port: u16) -> IMAPStream {
    IMAPStream {
      host: host,
      port: port,
      socket: None,
      connected: false,
      authenticated: false
    }
  }
  
  // build connection to host/port(IMAPServer)
  pub fn connect(&mut self) {
    match TcpStream::connect(self.host, self.port) {
      Ok(stream) => {
        self.connected = true;
        self.socket = Some(stream);
        match read_to_end(self.socket.unwrap()) {
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
  }

}

#[inline]
fn read_to_end(mut stream: TcpStream) -> Result<String, Vec<u8>> {
  let mut bufs: Vec<u8> = Vec::new();
  let mut tryClose = false;
  loop {
    let mut buf = [0];
    stream.read(buf);
    bufs.push(buf[0]);

    // check CLRL, if yes then break
    if tryClose && buf[0] == 0x0a {
      break;
    }
    tryClose = buf[0] == 0x0d;
  }
  String::from_utf8(bufs)
}

#[test]
fn create_new_imap_stream() {
  let mut imapstream = IMAPStream::new("imap.qq.com", 143);
  assert!(imapstream.host == "imap.qq.com");
  assert!(imapstream.port == 143);
  imapstream.connect();
}