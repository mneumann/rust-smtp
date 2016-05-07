
fn ascii_upcase(ascii: u8) -> u8 {
    if ascii >= b'a' && ascii <= b'z' {
        ascii - b'a' + b'A'
    } else {
        ascii
    }
}

#[test]
fn test_ascii_upcase() {
    assert!(ascii_upcase(b'E') == b'E');
    assert!(ascii_upcase(b'e') == b'E');
}

fn ascii_upcase_compare(str: &[u8], against: &[u8]) -> bool {
    str.len() == against.len() &&
    str.iter().zip(against.iter()).all(|(&a,&b)| ascii_upcase(a) == ascii_upcase(b))
}

#[test]
fn test_ascii_upcase_compare() {
    assert!(ascii_upcase_compare(b"ehlo", b"EHLO") == true);
    assert!(ascii_upcase_compare(b"EHLO", b"EHLO") == true);
    assert!(ascii_upcase_compare(b"bHLO", b"EHLO") == false);
    assert!(ascii_upcase_compare(b"EHLO ", b"EHLO") == false);
}

#[derive(PartialEq, Eq)]
pub enum SmtpCommand {
    Ehlo,
    Helo,
    Mail,
    Rcpt,
    Data,
    Unknown
}

pub fn parse_line(line: &[u8]) -> SmtpCommand {
    if line.len() >= 4 {
        let cmd = &line[0..4];

        if ascii_upcase_compare(cmd, b"EHLO") {
            SmtpCommand::Ehlo
        }
        else if ascii_upcase_compare(cmd, b"HELO") {
            SmtpCommand::Helo
        }
        else if ascii_upcase_compare(cmd, b"MAIL") {
            SmtpCommand::Mail
        }
        else if ascii_upcase_compare(cmd, b"RCPT") {
            SmtpCommand::Rcpt
        }
        else if ascii_upcase_compare(cmd, b"DATA") {
           SmtpCommand::Data
        }
        else {
           SmtpCommand::Unknown
        }
    }
    else {
        SmtpCommand::Unknown
    }
}

#[test]
fn test_parse_commands() {
    assert!(parse_line(b"ehlo mail.ntecs.de\r\n") == SmtpCommand::Ehlo);
    assert!(parse_line(b"helo mail.ntecs.de\r\n") ==SmtpCommand:: Helo);
    assert!(parse_line(b"DATA\r\n") == SmtpCommand::Data);
}
