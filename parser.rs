#[feature(macro_rules)];

use std::result::{Result,Ok,Err};

#[deriving(Eq,Show)]
enum SmtpCommand<'a> {
  MAIL(&'a str),
  RCPT(&'a str),
  DATA,
}

#[deriving(Eq,Show)]
enum ParseError {
  SyntaxError(&'static str),
  InvalidLineEnding,
  UnknownCommand,
}

fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
  if a.len() == b.len() {
    a.to_ascii().eq_ignore_case(b.to_ascii())
  }
  else {
    false
  }
}

struct SliceReader<'a> {
  data: &'a[u8]
}

impl<'a> SliceReader<'a> {

  fn new(data: &'a[u8]) -> SliceReader<'a> {
      SliceReader { data: data }
  }

  fn len(&self) -> uint { self.data.len() }
  fn is_empty(&self) -> bool { self.len() == 0 }
  fn data(&self) -> &'a[u8] { self.data }

  /// Remove `n` (but no more than len()) items from the back and return them.
  fn pop_back(&mut self, n: uint) -> &'a[u8] {
    if n > self.len() { debug!("pop_back(): n > len"); }
    let n = std::cmp::min(n, self.len());
    let (front, back) = self.split_at(self.len() - n);
    self.data = front;
    return back;
  }

  /// Remove `n` (but no more than len()) items from the front and return them. 
  fn pop_front(&mut self, n: uint) -> &'a[u8] {
    if n > self.len() { debug!("pop_front(): n > len"); }
    let n = std::cmp::min(n, self.len());

    let (front, back) = self.split_at(n);
    self.data = back;
    return front;
  }

  fn pop_while(&mut self, cond: |u8| -> bool) -> &'a[u8] {
    let mut cnt = 0;
    for &b in self.data.iter() {
      if cond(b) {
        cnt += 1;
      } else {
        break;
      }
    }
    self.pop_front(cnt)
  }

  fn split_at(&self, pos: uint) -> (&'a[u8], &'a[u8]) {
    assert!(pos <= self.data.len());
    (self.data.slice(0, pos), self.data.slice(pos, self.data.len()))
  }
}


fn parse_command<'a>(line: &'a[u8]) -> Result<SmtpCommand<'a>, ParseError>  {
    let mut line = SliceReader::new(line);

    if line.len() < 2 {
      return Err(InvalidLineEnding);
    }

    let crlf = line.pop_back(2);
    if crlf != bytes!("\r\n") {
      return Err(InvalidLineEnding);
    }

    let cmd = line.pop_front(4);
    if ascii_eq_ignore_case(cmd, bytes!("MAIL")) {
        if line.pop_while(|b| b == (' ' as u8) ).len() == 0 {
            return Err(SyntaxError("Invalid MAIL command: Missing SP"));
        }

        if ascii_eq_ignore_case(line.pop_front(5), bytes!("FROM:")) {
            let addr = line.pop_while(|b| b != (' ' as u8));
            if line.is_empty() {
                // XXX: Verify mail addr
                Ok(MAIL(std::str::from_utf8(addr).unwrap()))
            }
            else {
                Err(SyntaxError("Invalid MAIL command"))
            }
        }
        else {
            Err(SyntaxError("Invalid MAIL command"))
        }
    }
    else if ascii_eq_ignore_case(cmd, bytes!("DATA")) {
      if line.is_empty() {
          Ok(DATA)
      }
      else {
          Err(SyntaxError("Invalid DATA command"))
      }
    }
    else {
      Err(UnknownCommand)
    }
}

macro_rules! assert_match(
    ($given:expr , $pattern:pat) => ({
        let given_val = $given;
        match given_val {
          $pattern => {}
          _ => {
             fail!("assertion failed: `value does not match pattern`"); 
          }
        }
    })
)

macro_rules! test_parse_command (
  ($str:expr, $pat:pat) => ( 
    match bytes!($str) {
      cmd => assert_match!(parse_command(cmd), $pat)
    }
  )
)

#[test]
fn test_commands() {
  //test_parse_command!("", Err(InvalidLineEnding));
  test_parse_command!("\r", Err(InvalidLineEnding));
  test_parse_command!("\n", Err(InvalidLineEnding));
  test_parse_command!("\n\r", Err(InvalidLineEnding));
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>", Err(InvalidLineEnding));
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r", Err(InvalidLineEnding));
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\n", Err(InvalidLineEnding));
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\n\r", Err(InvalidLineEnding));

  test_parse_command!("MAIL FROM:<mneumann@ntecs.de blah\r\n", Err(SyntaxError("Invalid MAIL command")));
                
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r\n", Ok(MAIL("<mneumann@ntecs.de>")));


  test_parse_command!("DATA\r\n", Ok(DATA));
  test_parse_command!("data\r\n", Ok(DATA));
  test_parse_command!("data test\r\n", Err(SyntaxError("Invalid DATA command")));
}

fn main() {
  let buf = bytes!("MAIL FROM:<mneumann@ntecs.de>\r\n");
  let cmd = parse_command(buf);
  println!("{}", cmd);
}
