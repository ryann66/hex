use std::{process::exit, vec::Vec, io};

use queues::{CircularBuffer, IsQueue};

enum ReadMode {
	Binary,
	Decimal,
	Hex,
	Octal,
	Interpret
}

enum WriteMode {
	Binary,
	Decimal,
	Hex,
	Octal
}

enum WriteLength {
	Unfixed,
	Fixed(u64)
}

fn print_help() {
	println!("help me");
}

/**
   Converts the given argument into the specified format and returns either the converted string or an error message
 */
fn convert(ref arg: String, read_mode: &ReadMode, write_mode: &WriteMode, write_length: &WriteLength, signed_mode: bool) -> Result<String, String> {
	Ok(arg.to_string())
}

/**
   Converts numbers into different representations
 */
fn main() {
	// get args in a queue, use queue capacity to trim first arg
	let mut args = CircularBuffer::new(std::env::args().len() - 1);
	for arg in std::env::args() {
		let _ = args.add(arg);
	}

	// set standard settings
	let mut read_mode = ReadMode::Interpret;
	let mut write_mode = WriteMode::Hex;
	let mut write_length = WriteLength::Unfixed;
	let mut signed_mode = false;

	// save space for the results of conversions to be stored in
	let mut results = CircularBuffer::new(std::env::args().len() - 1);

	// process all the args
	while let Ok(arg) = args.remove() {
		match arg.chars().collect::<Vec<char>>()[..] {
			['-', 'd'] => {
				write_mode = WriteMode::Decimal;
			}
			['-', 'b'] => {
				write_mode = WriteMode::Binary;
			}
			['-', 'o'] => {
				write_mode = WriteMode::Octal;
			}
			['-', 'x'] => {
				write_mode = WriteMode::Hex;
			}
			['-', 'D'] => {
				read_mode = ReadMode::Decimal;
			}
			['-', 'B'] => {
				read_mode = ReadMode::Binary;
			}
			['-', 'O'] => {
				read_mode = ReadMode::Octal;
			}
			['-', 'X'] => {
				read_mode = ReadMode::Hex;
			}
			['-', 'F'] => {
				read_mode = ReadMode::Interpret;
			}		
			['-', 'u'] => {
				signed_mode = false;
			}
			['-', 's'] => {
				signed_mode = true;
			}
			['-', 'h'] | ['-', 'H'] | ['-', '?'] => {
				print_help();
				exit(if args.size() > 0 { 1 } else { 0 });
			}
			['-', 'w', '=', ..] => {
				// width
				let (_, num) = arg.split_at(3);
				if let Ok(intnum) = num.parse::<u64>() {
					write_length = WriteLength::Fixed(intnum);
				} else {
					println!("Error! unrecognizable option: {}", arg);
					exit(1);
				}
			}
			_ => {
				// something else (assume number)
				match convert(arg, &read_mode, &write_mode, &write_length, signed_mode) {
					Ok(str) => {
						let _ = results.add(str);
					}
					Err(str) => {
						println!("Error! {}", str);
						exit(1);
					}
				};
			}
		};
	}

	// print any results calculated so far
	if results.size() > 0 {
		while let Ok(res) = results.remove() {
			println!("{}", res);
		}
		exit(0);
	}

	// convert and print numbers as they come in from stdin
	loop {
		let mut line = String::new();
		match io::stdin().read_line(&mut line) {
			Ok(0) => {
				// eof
				exit(0);
			}
			Ok(_) => {
				// presumed number
				let _ = line.split_off(line.len() - 2);
				match convert(line, &read_mode, &write_mode, &write_length, signed_mode) {
					Ok(str) => {
						println!("{}", str);
					}
					Err(str) => {
						println!("Error! {}", str);
						exit(1);
					}
				};
			}
			Err(_) => {
				// some read error
				exit(1);
			}
		}
	}
}
