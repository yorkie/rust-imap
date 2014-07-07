
rust-imap
================
IMAP client for Rust

### Installation

Add imap.rs via your `Cargo.toml`:
```toml
[dependencies.imap]
git = "https://github.com/yorkie/imap.rs"
```

### Usage
```rs
extern crate imap;
use imap::IMAPStream;

let mut imapstream = box IMAPStream::new("imap.qq.com", 143);
imapstream.connect();
imapstream.login("username", "password");
imapstream.select("inbox");
imapstream.logout();
```

### License

MIT
