#[warn(unused_must_use)];

use std::io::IoResult;
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

fn read_line(io: &mut BufferedStream<TcpStream>) -> ~str {
    match io.read_line() {
        Ok(mut line) => {
            line.pop_char(); // XXX: assert '\r' 
            line.pop_char(); // XXX: assert '\r' 
            line
        }
        _ => fail!("Invalid line")
    }
}

/*
fn read_data_end(io: &mut BufferedStream<TcpStream>) -> ~[u8] {
    let mut buf: ~[u8] = ~[];
    let ends_with = bytes!("\r\n.\r\n");

    match io.read_byte() {
        Ok(byte) => {
           buf.push(byte);
           if buf.len() >= ends_with.len() {
               if buf.slice_from(buf.len() - ends_with.len()) == ends_with {
                   return buf;
               }
           }
        }
        _ => fail!()
    }
}
*/

enum Command {
    HELO(~str),
    EHLO(~str),
    FROM(~str), // MAIL FROM
    TO(~str), // RCPT TO
    DATA,
    QUIT,
    Invalid
}

fn read_expect(io: &mut BufferedStream<TcpStream>, expect: &[u8]) -> bool {
    for &byte in expect.iter() {
        if read_ascii(io) != byte as char {
            return false
        }
    }
    return true;
}

fn read_command(io: &mut BufferedStream<TcpStream>) -> Command {
    match read_ascii(io) {
        'H' => {
            if read_expect(io, bytes!("ELO ")) { HELO(read_line(io)) }
            else { Invalid }
        }
        'E' => {
            if read_expect(io, bytes!("HLO ")) { EHLO(read_line(io)) }
            else { Invalid }
        }
        'M' => {
            if read_expect(io, bytes!("AIL FROM:")) { FROM(read_line(io)) }
            else { Invalid }
        }
        'R' => {
            if read_expect(io, bytes!("CPT TO:")) { TO(read_line(io)) }
            else { Invalid }
        }
        'D' => {
            if read_expect(io, bytes!("ATA\r\n")) { DATA }
            else { Invalid }
        }
        'Q' => {
            if read_expect(io, bytes!("UIT\r\n")) { QUIT }
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

    let mut client_hostname = ~"";

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
            FROM(mailfrom) => {
                println!("FROM: {}", mailfrom);
                io.write("250 Ok\r\n".as_bytes()); io.flush();
            }
            TO(mailto) => {
                println!("TO: {}", mailto);
                io.write("250 Ok\r\n".as_bytes()); io.flush();
            }
            DATA => {
                println!("DATA");
                io.write("354 End data with <CR><LF>.<CR><LF>\r\n".as_bytes()); io.flush();
                loop {
                    let line = read_line(&mut io);
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
                break; // XXX make sure to close the connection
            }
            _ => {
                fail!()
            }
        }
    }

    debug!("End handling connection");
}


fn main() {
    let addr: SocketAddr = from_str("127.0.0.1:2525").unwrap();

    match TcpListener::bind(addr) {
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
