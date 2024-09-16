use std::{process::exit, vec::Vec, io};

use queues::{CircularBuffer, IsQueue};
use bitvec::prelude::BitVec;

#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum ReadMode {
	Binary,
	Decimal,
	Hex,
	Octal,
	Interpret
}

#[derive(Clone, Copy)]
enum WriteMode {
	Binary,
	Decimal,
	Hex,
	Octal
}

#[derive(Clone, Copy)]
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

/*
   A note on integer representation using bitvec.  All integers are stored 'little-endian'
   where the the least significant bit gets the lowest address.  Hence,
   bitvec[0] is the 1s position and bitvec[2] would be the 4s position
 */

 /**
    adds the number represented by the addend to the number represented by sum
  */
fn add_bits_to(sum:&mut BitVec, addend:&BitVec) {
	todo!()
}

/**
   multiplies the number represented by bits by ten
 */
fn multiply_by_ten(bits:&mut BitVec) {
	todo!()
}

/**
   Attempts to parse the string arg into an integer
   The result integer is returned as a little-endian integer (signedness indicated by signed_mode)
   On failure, returns an Err with an error message
 */
#[inline]
fn read(arg: &String, mut read_mode: ReadMode, signed_mode: bool) -> Result<BitVec, String> {
	let mut negative_arg = false;
	// strip all prefixes from the arg and interpret
	let stripped_arg = {
		// strip negative sign from the arg
		let positive_arg = {
			if let Some(tmp_arg) = arg.strip_prefix('-') {
				negative_arg = true;
				tmp_arg
			} else {
				arg
			}
		};

		// strip prefix depending on read_mode
		match read_mode {
			ReadMode::Binary => {
				if let Some(tmp_arg) = positive_arg.strip_prefix("0b") {
					tmp_arg
				} else {
					positive_arg
				}
			}
			ReadMode::Hex => {
				if let Some(tmp_arg) = positive_arg.strip_prefix("0x") {
					tmp_arg
				} else {
					positive_arg
				}
			}
			ReadMode::Octal => {
				if let Some(tmp_arg) = positive_arg.strip_prefix("0o") {
					tmp_arg
				} else {
					positive_arg
				}
			}
			ReadMode::Decimal => {
				positive_arg
			}
			ReadMode::Interpret => {
				if let Some(tmp_arg) = positive_arg.strip_prefix("0b") {
					read_mode = ReadMode::Binary;
					tmp_arg
				} else if let Some(tmp_arg) = positive_arg.strip_prefix("0x") {
					read_mode = ReadMode::Hex;
					tmp_arg
				} else if let Some(tmp_arg) = positive_arg.strip_prefix("0o") {
					read_mode = ReadMode::Octal;
					tmp_arg
				} else if positive_arg.contains(['a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F']) {
					read_mode = ReadMode::Hex;
					positive_arg
				} else {
					read_mode = ReadMode::Decimal;
					positive_arg
				}
			}
		}
	};
	
	// check negative arguments make sense
	if !signed_mode && negative_arg {
		return Err("Negative numbers not allowed in unsigned mode".to_string());
	}
	if read_mode != ReadMode::Decimal && negative_arg {
		return Err("- operator is only allowed with decimal numbers".to_string());
	}

	let mut bits = BitVec::new();
	match read_mode {
		ReadMode::Binary => {
			for c in stripped_arg.chars() {
				match c {
					'0' => bits.push(false),
					'1' => bits.push(true),
					c => return Err(format!("Character {} not allowed in binary numbers", c))
				};
			};
		}
		ReadMode::Octal => {
			for c in stripped_arg.chars() {
				match c {
					'0' => {
						bits.push(false);
						bits.push(false);
						bits.push(false);
					}
					'1' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'2' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'3' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
					}
					'4' => {
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'5' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'6' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'7' => {
						bits.push(true);
						bits.push(true);
						bits.push(true);
					}
					c => return Err(format!("Character {} not allowed in octal numbers", c))
				};
			};
		}
		ReadMode::Hex => {
			for c in stripped_arg.chars() {
				match c {
					'0' => {
						bits.push(false);
						bits.push(false);
						bits.push(false);
						bits.push(false);
					}
					'1' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
						bits.push(false);
					}
					'2' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'3' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'4' => {
						bits.push(false);
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'5' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'6' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
						bits.push(false);
					}
					'7' => {
						bits.push(true);
						bits.push(true);
						bits.push(true);
						bits.push(false);
					}
					'8' => {
						bits.push(false);
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'9' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'A' | 'a' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'B' | 'b' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'C' | 'c' => {
						bits.push(false);
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'D' | 'd' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'E' | 'e' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
						bits.push(true);
					}
					'F' | 'f' => {
						bits.push(true);
						bits.push(true);
						bits.push(true);
						bits.push(true);
					}
					c => return Err(format!("Character {} not allowed in hexadecimal numbers", c))
				};
			};
		}
		ReadMode::Decimal => {
			for c in stripped_arg.chars() {
				multiply_by_ten(&mut bits);
				let mut addend = BitVec::new();
				match c {
					'0' => todo!(),
					'1' => {
						addend.push(true);
					}
					'2' => {
						addend.push(false);
						addend.push(true);
					}
					'3' => {
						addend.push(true);
						addend.push(true);
					}
					'4' => {
						addend.push(false);
						addend.push(false);
						addend.push(true);
					}
					'5' => {
						addend.push(true);
						addend.push(false);
						addend.push(true);
					}
					'6' => {
						addend.push(false);
						addend.push(true);
						addend.push(true);
					}
					'7' => {
						addend.push(true);
						addend.push(true);
						addend.push(true);
					}
					'8' => {
						addend.push(false);
						addend.push(false);
						addend.push(false);
						addend.push(true);
					}
					'9' => {
						addend.push(true);
						addend.push(false);
						addend.push(false);
						addend.push(true);
					}
					c => return Err(format!("Character {} not allowed in decimal numbers", c))
				};
				add_bits_to(&mut bits, &addend);
			}
		}
		ReadMode::Interpret => assert!(false)
	};
	
	return Ok(bits);
}

/**
   Converts the stream of bits representing a little-endian integer (signedness indicated by signed_mode) into
   a string version of the integer in the format given by write_mode
 */
#[inline]
fn write(bits: &BitVec, write_mode: WriteMode, write_length: WriteLength, signed_mode: bool) -> String {
	String::new()
}

/**
   Converts the given argument into the specified format and returns either the converted string or an error message
 */
#[inline]
fn convert(ref arg: &String, read_mode: ReadMode, write_mode: WriteMode, write_length: WriteLength, signed_mode: bool) -> Result<String, String> {
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
				match convert(&arg, read_mode, write_mode, write_length, signed_mode) {
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
				match convert(&line, read_mode, write_mode, write_length, signed_mode) {
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
