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

fn parse_command<'a>(line: &'a[u8]) -> Result<SmtpCommand<'a>, ParseError>  {
    if line.len() < 2 {
      return Err(InvalidLineEnding);
    }

    let crlf = line.slice(line.len() - 2, line.len());
    if crlf != bytes!("\r\n") {
      return Err(InvalidLineEnding);
    }

    let line = line.initn(2); // drop off line ending

    let cmd = line.slice(0, 4);
    if ascii_eq_ignore_case(cmd, bytes!("MAIL")) {
        if ascii_eq_ignore_case(line.slice(4, 11), bytes!(" FROM:<")) {
            let addr = line.slice_from(11).init();
            let rem = line.slice_from(line.len() - 1);
            if rem == bytes!(">") {
                // XXX: Verify mail addr
                Ok(MAIL(std::str::from_utf8(addr).unwrap()))
            }
            else {
                Err(SyntaxError("Invalid MAIL command: Missing >"))
            }
        }
        else {
            Err(SyntaxError("Invalid MAIL command"))
        }
    }
    else if ascii_eq_ignore_case(cmd, bytes!("DATA")) {
      if line.slice_from(4).len() == 0 {
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

  test_parse_command!("MAIL FROM:<mneumann@ntecs.de\r\n", Err(SyntaxError("Invalid MAIL command: Missing >")));
                
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r\n", Ok(MAIL("mneumann@ntecs.de")));


  test_parse_command!("DATA\r\n", Ok(DATA));
  test_parse_command!("data\r\n", Ok(DATA));
  test_parse_command!("data test\r\n", Err(SyntaxError("Invalid DATA command")));
}

fn main() {
  let buf = bytes!("MAIL FROM:<mneumann@ntecs.de>\r\n");
  let cmd = parse_command(buf);
  println!("{}", cmd);
}
