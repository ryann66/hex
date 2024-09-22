use std::{io, process::exit, vec::Vec};

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
	RoundUp,
	Fixed(u64)
}

enum WriteSeparator {
	Separator(String),
	RuntimeDetermine(Option<String>),
	None
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
	println!("-f Sets the width of ouput to the minimum number of characters to represent the number");
	println!("-r Rounds the width of output to a pretty length");
	println!("	Octal and binary will be rounded to bytes, octal will be rounded to even lengths");
	println!("	This option does not effect the print length of decimal numbers");
	println!("-c[=<sep>] Adds a separator character between groups of digits");
	println!("	Separator is added every 3 chars for decimal, 2 for hex, 8 for binary, and 2 for octal");
	println!("	Default separator is ',' for decimal and ' ' for everything else");
	println!("-t Removes the separator character");
}

/*
   A note on integer representation using bitvec.  All integers are stored 'little-endian'
   where the the least significant bit gets the lowest address.  Hence,
   bitvec[0] is the 1s position and bitvec[2] would be the 4s position
 */

/*
   Negates the integer represented by the bitvec
*/
fn negate(bits: &mut BitVec) {
	todo!()
}

/**
   Attempts to parse the string arg into an integer
   The result integer is returned as a little-endian integer (signedness indicated by signed_mode)
   On failure, returns an Err with an error message
 */
#[inline]
fn read(arg: &String, mut read_mode: ReadMode, write_mode: WriteMode, write_length: WriteLength, signed_mode: bool) -> Result<BitVec, String> {
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
			// convert argument as a string
			// process:
			//	check string num can be divided by 2
			//		if yes: divide by two, add a divide to the operation stack
			//		if no: subtract by one, add a subtract to the operation stack
			//	iterate through operation stack
			//		if subtract: set the lsb to true (1)
			//			note: this is the same as adding one because we cannot get two subtracts in a row because
			//				this would indicate that there exists a number x such that x is odd and x + 1 is odd
			//		if divide: left shift the whole number by 1, leaving a 0 at the lsb

			enum Operation {
				Divide,
				Subtract
			}

			/*
			   returns if the integer that the string would parse into is even
			 */
			fn string_is_even(str: &Vec<char>) -> bool {
				match str.last() {
					Some('0') | Some('2') | Some('4') | Some('6') | Some('8') => true,
					_ => false
				}
			}

			/*
			   subtracts one from the integer represented by the string
			 */
			fn sub_string(str: &mut Vec<char>) {
				// subtract off numbers
				for index in (0..str.len()).rev() {
					let ch = match str[index] {
						'9' => '8',
						'8' => '7',
						'7' => '6',
						'6' => '5',
						'5' => '4',
						'4' => '3',
						'3' => '2',
						'2' => '1',
						'1' => '0',
						'0' => '9',
						_ => panic!()
					};
					str[index] = ch;
					if ch != '9' { break; };
				};

				// trim leading zeroes
				while str.get(0) == Some(&'0') {
					str.remove(0);
				};

			}

			/*
			   divides the integer represented by the string by 2
			 */
			fn div_string(str: &mut Vec<char>) {
				// divide numbers starting in the front
				let mut carry = false;
				for index in 0..str.len() {
					str[index] = match str[index] {
						'9' => {
							if carry {
								'9'
							} else {
								carry = true;
								'4'
							}
						}
						'8' => {
							if carry {
								carry = false;
								'9'
							} else {
								'4'
							}
						}
						'7' => {
							if carry {
								'8'
							} else {
								carry = true;
								'3'
							}
						}
						'6' => {
							if carry {
								carry = false;
								'8'
							} else {
								'3'
							}
						}
						'5' => {
							if carry {
								'7'
							} else {
								carry = true;
								'2'
							}
						}
						'4' => {
							if carry {
								carry = false;
								'7'
							} else {
								'2'
							}
						}
						'3' => {
							if carry {
								'6'
							} else {
								carry = true;
								'1'
							}
						}
						'2' => {
							if carry {
								carry = false;
								'6'
							} else {
								'1'
							}
						}
						'1' => {
							if carry {
								'5'
							} else {
								carry = true;
								'0'
							}
						}
						'0' => {
							if carry {
								carry = false;
								'5'
							} else {
								'0'
							}
						}
						_ => panic!()
					};
				};

				// trim leading zeroes
				while str.get(0) == Some(&'0') {
					str.remove(0);
				};
			}
			
			// parse through input string
			let mut str: Vec<char> = arg.chars().collect();
			let mut ops = Vec::new();
			while !str.is_empty() {
				if string_is_even(&str) {
					div_string(&mut str);
					ops.push(Operation::Divide);
				} else {
					sub_string(&mut str);
					ops.push(Operation::Subtract);
				};
			};
			
			// apply operation sequence to bits
			while let Some(op) = ops.pop() {
				match op {
					Operation::Divide => {
						bits.push(false);
					}
					Operation::Subtract => {
						let _ = bits.pop();
						bits.push(true);
					}
				};
			}
		}
		ReadMode::Interpret => panic!()
	};

	// increase length of bits to write_length
	let target_len = match write_length {
		WriteLength::Unfixed => bits.len() as u64,
		WriteLength::RoundUp => {
			let min_len = bits.len() as u64;
			let int = match write_mode {
				WriteMode::Binary | WriteMode::Hex => 8u64,
				WriteMode::Octal => 6u64,
				WriteMode::Decimal => 1u64
			};
			min_len.next_multiple_of(int)
		}
		WriteLength::Fixed(len) => len
	};
	if (bits.len() as u64) > target_len {
		return Err("Number unrepresentable in fixed width".to_string());
	}
	while (bits.len() as u64) < target_len {
		bits.insert(0, false);
	}

	// flip bits and add one if reading from decimal and negative
	if negative_arg && read_mode == ReadMode::Decimal {
		negate(&mut bits);
	}
	
	return Ok(bits);
}

/**
   Converts the stream of bits representing a little-endian integer (signedness indicated by signed_mode) into
   a string version of the integer in the format given by write_mode
 */
#[inline]
fn write(bits: &BitVec, write_mode: WriteMode, write_separator: &WriteSeparator, signed_mode: bool) -> String {
	println!("{}", bits);
	todo!()
}

/**
   Converts the given argument into the specified format and returns either the converted string or an error message
 */
fn convert(ref arg: &String, read_mode: ReadMode, write_mode: WriteMode, write_length: WriteLength, write_separator: &mut WriteSeparator, signed_mode: bool) -> Result<String, String> {
	// runtime fix write_separator
	if let WriteSeparator::RuntimeDetermine(_) = write_separator {
		*write_separator = WriteSeparator::Separator(match write_mode {
			WriteMode::Decimal => ',',
			WriteMode::Binary | WriteMode::Octal | WriteMode::Hex => ' '
		}.to_string());
	}
	
	// do conversion
	match read(arg, read_mode, write_mode, write_length, signed_mode) {
		Ok(bits) => {
			Ok(write(&bits, write_mode, &write_separator, signed_mode))
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
	let mut write_separator = WriteSeparator::None;
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
					println!("Error! Unrecognizable option: {}", arg);
					exit(1);
				}
			}
			['-', 'f'] => {
				write_length = WriteLength::Unfixed;
			}
			['-', 'r'] => {
				write_length = WriteLength::RoundUp;
			}
			['-', 'c', '=', ..] => {
				// separator
				let (_, sep) = arg.split_at(3);
				if sep.is_empty() {
					println!("Error! Empty separator!");
					exit(1);
				}
				write_separator = WriteSeparator::Separator(sep.to_string());
			}
			['-', 'c'] => {
				write_separator = WriteSeparator::RuntimeDetermine(None);
			}
			['-', 't'] => {
				write_separator = WriteSeparator::None;
			}
			['-', 'v'] | ['-', 'V'] => {
				println!("Hex v{}", env!("CARGO_PKG_VERSION"));
				exit(0);
			}
			_ => {
				// something else (assume number)
				match convert(&arg, read_mode, write_mode, write_length, &mut write_separator, signed_mode) {
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
				match convert(&line, read_mode, write_mode, write_length, &mut write_separator, signed_mode) {
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
