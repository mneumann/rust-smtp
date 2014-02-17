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
    let cmd = line.slice(0, 4);
    if ascii_eq_ignore_case(cmd, bytes!("MAIL")) {
        if ascii_eq_ignore_case(line.slice(4, 11), bytes!(" FROM:<")) {
            let addr = line.slice(11, line.len() - 3);
            let rem = line.slice_from(line.len() - 3);
            if rem == bytes!(">\r\n") {
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

/*
macro_rules! assert_eq_parse_command (
  ($str:expr, $exp:expr) => ( 
    match bytes!($str) {
      cmd => assert_eq!(parse_command(cmd), $exp)
    }
  )
)
*/

macro_rules! test_parse_command (
  ($str:expr, $pat:pat) => ( 
    match bytes!($str) {
      cmd => assert_match!(parse_command(cmd), $pat)
    }
  )
)

#[test]
fn test_commands() {
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r\n", Ok(MAIL("mneumann@ntecs.de")));
  test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r", Err(_));
}

fn main() {
  let buf = bytes!("MAIL FROM:<mneumann@ntecs.de>\r\n");
  let cmd = parse_command(buf);
  println!("{}", cmd);
}
