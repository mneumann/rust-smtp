parser: parser.rs
	rustc parser.rs

test_parser: parser.rs
	rustc -o test_parser --test parser.rs

test: test_parser
	./test_parser

smtp: smtp.rs
	rustc smtp.rs
