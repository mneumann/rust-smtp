#![warn(unused_must_use)]

#[macro_use] 
extern crate log;

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::net::{TcpListener,TcpStream};
use std::thread::spawn;

fn read_ascii_char(io: &mut Read) -> Result<u8> {
    let buf = &mut vec![0; 1];
    try!(io.read_exact(buf));
    if buf[0] < 127 {
        return Ok(buf[0]);
    }
    Err(Error::new(ErrorKind::InvalidData, "Non-ASCII char(s) detected"))
}

// RFC 5321 Section 2.3.8. Lines
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;
fn read_line(io: &mut Read) -> Result<String> {
    let mut s = "".to_string();

    loop {
        match try!(read_ascii_char(io)) {
            CR   => { break }
            LF   => { return Err(Error::new(ErrorKind::InvalidInput, "Expected CR (before LF). Got LF")) }
            byte => { s.push(byte as char); }
        }
    }

    if try!(read_ascii_char(io)) == (LF as u8) {
        Ok(s)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "LF expected after CR"))
    }
}

fn read_expect(io: &mut Read, expect: &[u8]) -> bool {
    let buf = &mut vec![0; expect.len()];
    if io.read_exact(buf).is_ok() {
        for &byte in expect.iter() {
            if let Ok(b) = read_ascii_char(io) {
                if b != byte {
                    return false
                }
            }
        }
        return true;
    }
    return false;
}

#[derive(PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
enum Command {
    HELO(String),
    EHLO(String),
    MAIL_FROM(String),
    RCPT_TO(String),
    DATA,
    QUIT,
    Invalid
}

fn read_command(io: &mut Read) -> Result<Command> {
    match try!(read_ascii_char(io)) as char {
        'H' => {
            if read_expect(io, b"ELO ") { Ok(Command::HELO(read_line(io).unwrap())) }
            else { Ok(Command::Invalid) }
        }
        'E' => {
            if read_expect(io, b"HLO ") { Ok(Command::EHLO(read_line(io).unwrap())) }
            else { Ok(Command::Invalid) }
        }
        'M' => {
            if read_expect(io, b"AIL FROM:") { Ok(Command::MAIL_FROM(read_line(io).unwrap())) }
            else { Ok(Command::Invalid) }
        }
        'R' => {
            if read_expect(io, b"CPT TO:") { Ok(Command::RCPT_TO(read_line(io).unwrap())) }
            else { Ok(Command::Invalid) }
        }
        'D' => {
            if read_expect(io, b"ATA\r\n") { Ok(Command::DATA) }
            else { Ok(Command::Invalid) }
        }
        'Q' => {
            if read_expect(io, b"UIT\r\n") { Ok(Command::QUIT) }
            else { Ok(Command::Invalid) }
        }
        _ => {
            Ok(Command::Invalid)
        }
    }
}

fn handle_connection(mut conn: TcpStream) {
    debug!("Got connection");

    let server_hostname = "mail.ntecs.de";
    let server_agent = "rust-smtp";

    let response_220 = format!("220 {} ESMTP {}\r\n", server_hostname, server_agent);
    if let Err(_) = conn.write_all(&response_220.into_bytes()) {
        error!("Error while writing 220 hostname and agent response");
        return;        
    }

    let client_hostname = match read_command(&mut conn) {
        Ok(Command::EHLO(h)) => h,
        Ok(Command::HELO(h)) => h,
        Ok(unexpected) => {error!("Unexpected command {:?}", unexpected); return}
        Err(_) => {error!("IO error while reading command. Quitting"); return}
    };

    println!("Client hostname: {}", client_hostname);
    
    if let Ok(_) = conn.write_all(&format!("250 Hello {}\r\n", client_hostname).into_bytes()) {
        info!("Saying Hello to {}", client_hostname);
    }
    else {
        error!("Error while writing Hello. Quitting session.");
        return;
    }
    
    let mut bytes_to_write : Vec<u8> = Vec::new();
    loop {
        let cmd = read_command(&mut conn);
        match cmd {
            Ok(Command::MAIL_FROM(mailfrom)) => {
                println!("FROM: {}", mailfrom);
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter())
            },
            Ok(Command::RCPT_TO(mailto)) => {
                println!("TO: {}", mailto);
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter()); 
            },
            Ok(Command::DATA) => {
                println!("DATA");
                bytes_to_write.extend("354 End data with <CR><LF>.<CR><LF>\r\n".as_bytes().iter());
                loop {
                    let line = read_line(&mut conn).unwrap();
                    println!("Data|{}|", line);
                    if line.as_str() == "." {
                        println!("Got end");
                        break;
                    }
                }
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter());
            },
            Ok(Command::QUIT) => {
                println!("QUIT");
                bytes_to_write.extend("221 Bye\r\n".as_bytes().iter()); 
                break;
            },
            Ok(_) => {
                panic!("Unknown command {:?}", cmd)
            },
            Err(_) => {
                panic!("IO Error")  
            }
        };

        if let Ok(_) = conn.write_all(&bytes_to_write) {
            let flush_result = conn.flush();
            if !flush_result.is_ok() {
                error!("Failed to flush bytes to connection. Ending session");
                return;
            }
        }
        else {
            error!("Failed to write bytes. Ending session.");
            return;
        }
        
    }
}

fn main() {
    match TcpListener::bind(("127.0.0.1", 2525)) {
        Ok(listener) => {
            for acceptor in listener.incoming() {
                match acceptor {
                    Ok(conn) => { spawn(|| handle_connection(conn)); },
                    _ => error!("Could not accept connection.")
                }
            }
        }
        _ => { panic!() }
    }
}
