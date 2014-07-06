
extern crate imap;
use imap::IMAPStream;

fn main() {
  let mut imapstream = box IMAPStream::new("imap.qq.com", 143);
  imapstream.connect();
  imapstream.login("550532246@qq.com", "xxxxxx");
}