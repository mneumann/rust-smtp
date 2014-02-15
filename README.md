rust-smtp
=========

SMTP (Simple Mail Transfer Protocol) implementation for Rust.

## Goal

The goal of this project is to implement a [RFC 5321][RFC5321] compliant SMTP client and server library, which hopefully
will turn into a fully featured SMTP server one day. Rust is the ideal language to write such a project in, due to it's
safety features (no memory-holes, task-separation), it's efficiency, the asynchronous network implemention and the
high-level language features.

Making this a library makes it a lot more useful than a program, as it can be configured much easier and to a wider
degree. For example the way local users are looked up can be implemented in an arbitrary way; there is no need for
external lookup processes as found in [OpenSMTPd][opensmtpd].

## Todo

- [ ] Successfully parse a simple SMTP session
- [ ] Simple SMTP proxy, no delivery, no relaying
- [ ] Support SSL
- [ ] Support local delivery (on port 587)
- [ ] Support relaying (queueing)
- [ ] Add anti-spam features

[RFC5321]: http://tools.ietf.org/html/rfc5321
[opensmtpd]: http://www.opensmtpd.org/
