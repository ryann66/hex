use std::{process::exit, vec::Vec, io};

use queues::{CircularBuffer, IsQueue};
use bitvec::prelude::BitVec;

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

/**
   Prints the program help
 */
fn print_help() {
	println!("HEX");
    println!("Tool for converting between different number types");
    println!("Usage: hex <options> <params>");
    println!("Can take many params at once or leave empty to read from stdin");
    println!("");
    println!("Options:");
    println!("-h Displays this help and exits");
    println!("-v Displays the program version and exits");
    println!("-B Forces the program to read input as a a binary");
    println!("-D Forces the program to read input as a base 10 (decimal) integer");
    println!("-O Forces the program to read input as an octal");
    println!("-X Forces the program to read input as a hexadecimal");
    println!("-F Lets the program decide how to read input based off prefix (default)");
    println!("-b Writes output in binary with prefix");
    println!("-d Writes output in base 10 (decimal)");
    println!("-o Writes output in octal with prefix");
    println!("-x Writes output in hexadecimal with prefix");
    println!("-s Puts the system into signed mode (two's complement).  Use '_' for '-' in decimals");
    println!("-u Puts the system into unsigned mode (default)");
    println!("-w=<Num> Sets the width of output");
	println!("	With decimal and octal the length is in characters (excluding - sign)");
	println!("	With binary and hexadecimal the length is in bytes");
}

/**
   Attempts to parse the string arg into an integer
   The result integer is returned as a big endian integer (signedness indicated by signed_mode)
   On failure, returns an Err with an error message
 */
#[inline]
fn read(arg: &String, read_mode: &ReadMode, signed_mode: bool) -> Result<BitVec, String> {
	Ok(BitVec::new())
}

/**
   Converts the stream of bits representing a big-endian integer (signedness indicated by signed_mode) into
   a string version of the integer in the format given by write_mode
 */
#[inline]
fn write(bits: &BitVec, write_mode: &WriteMode, write_length: &WriteLength, signed_mode: bool) -> String {
	String::new()
}

/**
   Converts the given argument into the specified format and returns either the converted string or an error message
 */
#[inline]
fn convert(ref arg: String, read_mode: &ReadMode, write_mode: &WriteMode, write_length: &WriteLength, signed_mode: bool) -> Result<String, String> {
	match read(arg, read_mode, signed_mode) {
		Ok(bits) => {
			Ok(write(&bits, write_mode, write_length, signed_mode))
		}
		Err(msg) => {
			Err(msg)
		}
	}
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
				exit(0);
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
			['-', 'v'] | ['-', 'V'] => {
				println!("Hex v{}", env!("CARGO_PKG_VERSION"));
				exit(0);
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
