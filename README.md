rust-smtp
=========

SMTP (Simple Mail Transfer Protocol) implementation for Rust.

## Goal

The goal of this project is to implement a [RFC 5321][RFC5321] compliant SMTP client and server library, which hopefully
will turn into a fully featured SMTP server one day. Rust is the ideal language to write such a project in, for various
reasons:

1. It's a safe language. Buffer bounds checking, no memory holes, task-separation.
2. It's efficient like C/C++.
3. Light-weight threading and asynchronous I/O leads to a much simpler implementation, only using
   slightly more memory (for each tasks' stack) than a hand-rolled evented implementation.
4. Rust is a lot more high-level and cleaner than other comparative languages.

Making this a library makes it a lot more useful than a program, as it can be configured much easier and to a wider
degree. For example the way local users are looked up can be implemented in an arbitrary way; there is no need for
external lookup processes as found in [OpenSMTPd][opensmtpd]. Also by using Rust and it's libraries, it is from the
beginning on fully platform independent and not tied to a specific range of platforms.

## Todo

- [ ] Successfully parse a simple SMTP session
- [ ] Simple SMTP proxy, no delivery, no relaying
- [ ] Support SSL
- [ ] Support local delivery (on port 587)
- [ ] Support relaying (queueing)
- [ ] Add anti-spam features

[RFC5321]: http://tools.ietf.org/html/rfc5321
[opensmtpd]: http://www.opensmtpd.org/
