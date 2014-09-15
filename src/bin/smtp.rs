#![warn(unused_must_use)]
#![feature(phase)]

#[phase(plugin, link)] extern crate log;

use std::io::{IoResult,IoError,InvalidInput};
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Listener,Acceptor,Writer};
use std::io::BufferedStream;
use std::task;

fn read_ascii(io: &mut BufferedStream<TcpStream>) -> char {
    match io.read_byte() {
        Ok(byte) if byte < 127 => byte as char,
        _ => fail!("Invalid ASCII character")
    }
}

fn invalid_input(desc: &'static str) -> IoError {
    IoError {kind: InvalidInput, desc: desc, detail: None}
}

// RFC 5321 Section 2.3.8. Lines
static CR: u8 = 0x0D;
static LF: u8 = 0x0A;
fn read_line(io: &mut BufferedStream<TcpStream>) -> IoResult<String> {
    let mut s = "".to_string();

    loop {
        match try!(io.read_byte()) {
            CR   => { break }
            LF   => { return Err(invalid_input("CR expected before LF")) }
            byte => { s.push_char(byte as char); }
        }
    }

    if try!(io.read_byte()) == LF {
        Ok(s)
    } else {
        Err(invalid_input("LF expected after CR"))
    }
}

fn read_expect(io: &mut BufferedStream<TcpStream>, expect: &[u8]) -> bool {
    for &byte in expect.iter() {
        if read_ascii(io) != byte as char {
            return false
        }
    }
    return true;
}

enum Command {
    HELO(String),
    EHLO(String),
    MAIL_FROM(String),
    RCPT_TO(String),
    DATA,
    QUIT,
    Invalid
}

fn read_command(io: &mut BufferedStream<TcpStream>) -> Command {
    match read_ascii(io) {
        'H' => {
            if read_expect(io, b"ELO ") { HELO(read_line(io).unwrap()) }
            else { Invalid }
        }
        'E' => {
            if read_expect(io, b"HLO ") { EHLO(read_line(io).unwrap()) }
            else { Invalid }
        }
        'M' => {
            if read_expect(io, b"AIL FROM:") { MAIL_FROM(read_line(io).unwrap()) }
            else { Invalid }
        }
        'R' => {
            if read_expect(io, b"CPT TO:") { RCPT_TO(read_line(io).unwrap()) }
            else { Invalid }
        }
        'D' => {
            if read_expect(io, b"ATA\r\n") { DATA }
            else { Invalid }
        }
        'Q' => {
            if read_expect(io, b"UIT\r\n") { QUIT }
            else { Invalid }
        }
        _ => {
            Invalid
        }
    }
}

fn handle_connection(conn: TcpStream) {
    debug!("Got connection");

    let mut io = BufferedStream::new(conn);
    let server_hostname = "mail.ntecs.de";
    let server_agent = "rust-smtp";

    write!(&mut io, "220 {} ESMTP {}\r\n", server_hostname, server_agent);
    io.flush();

    let mut client_hostname = "".to_string();

    match read_command(&mut io) {
        EHLO(h) => client_hostname = h,
        HELO(h) => client_hostname = h,
        _ => fail!("Expected EHLO or HELO")
    }

    println!("Client hostname: {}", client_hostname);
    
    write!(&mut io, "250 Hello {}\r\n", client_hostname);
    io.flush();

    loop {
        let cmd = read_command(&mut io);
        match cmd {
            MAIL_FROM(mailfrom) => {
                println!("FROM: {}", mailfrom);
                io.write("250 Ok\r\n".as_bytes()); io.flush();
            }
            RCPT_TO(mailto) => {
                println!("TO: {}", mailto);
                io.write("250 Ok\r\n".as_bytes()); io.flush();
            }
            DATA => {
                println!("DATA");
                io.write("354 End data with <CR><LF>.<CR><LF>\r\n".as_bytes()); io.flush();
                loop {
                    let line = read_line(&mut io).unwrap();
                    println!("Data|{}|", line);
                    if line.as_slice() == "." {
                        println!("Got end");
                        break;
                    }
                }
                io.write("250 Ok\r\n".as_bytes()); io.flush();
            }
            QUIT => {
                println!("QUIT");
                io.write("221 Bye\r\n".as_bytes()); io.flush();
                break;
            }
            _ => {
                fail!()
            }
        }
    }

    // XXX make sure to close the connection
    debug!("End handling connection");
}

fn main() {
    match TcpListener::bind("127.0.0.1", 2525) {
        Ok(listener) => {
            match listener.listen() {
                Ok(ref mut acceptor) => {
                    loop {
                        match acceptor.accept() {
                            Ok(conn) => {
                                task::spawn(proc() handle_connection(conn))
                            }
                            _ => { fail!() }
                        }
                    }
                }
                _ => { fail!() }
            }
        }
        _ => { fail!() }
    }

}
