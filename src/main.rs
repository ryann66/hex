use std::{io, ops::BitXorAssign, process::exit, vec::Vec};

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

#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum WriteMode {
	Binary,
	Decimal,
	Hex(bool /* is uppercase */),
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
	RuntimeDetermine,
	None
}

/// Prints the program help
fn print_help() {
	println!("HEX");
    println!("Tool for converting between different number types");
    println!("Usage: hex <options> <params>");
    println!("Can take many params at once or be left empty to read from stdin");
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
	println!("	Default is to print uppercase hex; use -xl to force lowercase");
    println!("-s Puts the system into signed mode (two's complement).  Use '-' in decimals");
	println!("	It is recommended to combine this with '-w'");
    println!("-u Puts the system into unsigned mode (default)");
    println!("-w=<Num> Sets the length of output in bytes");
	println!("	When writing in octal uses a 6-bit byte. Has no effect when writing in decimal");
	println!("-f Sets the width of ouput to the minimum number of characters to represent the number");
	println!("-r Rounds the width of output to a pretty length (usually a byte boundary)");
	println!("	Octal and binary will be rounded to bytes, octal will be rounded to even lengths");
	println!("	This option does not effect the print length of decimal numbers");
	println!("-c[=<sep>] Adds a separator character between groups of digits");
	println!("	Separator is added every 3 chars for decimal, 2 for hex, 8 for binary, and 2 for octal");
	println!("	Default separator is ',' for decimal and ' ' for everything else");
	println!("-t Removes the separator character");
	println!("-p Write prefixes on all non-decimal numbers (default)");
	println!("-n Omit prefixes from all numbers");
}

/// Multiplies the value the integer represented by the bitvec by -1 using two's complement
fn negative(bits: &mut BitVec) {
	// negate
	for mut bit in bits.iter_mut() {
		bit.bitxor_assign(true);
	}

	// add one
	for mut bit in bits.iter_mut().rev() {
		bit.bitxor_assign(true);
		if *bit {
			break;
		}
	}
}

/// Attempts to parse the string arg into an integer
/// The result integer is returned as an integer stored in the bitvec (signedness indicated by signed_mode)
/// On failure, returns an Err with an error message
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
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'2' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'3' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'4' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'5' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'6' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
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
						bits.push(false);
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'2' => {
						bits.push(false);
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'3' => {
						bits.push(false);
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'4' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'5' => {
						bits.push(false);
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'6' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
						bits.push(false);
					}
					'7' => {
						bits.push(false);
						bits.push(true);
						bits.push(true);
						bits.push(true);
					}
					'8' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
						bits.push(false);
					}
					'9' => {
						bits.push(true);
						bits.push(false);
						bits.push(false);
						bits.push(true);
					}
					'A' | 'a' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
						bits.push(false);
					}
					'B' | 'b' => {
						bits.push(true);
						bits.push(false);
						bits.push(true);
						bits.push(true);
					}
					'C' | 'c' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
						bits.push(false);
					}
					'D' | 'd' => {
						bits.push(true);
						bits.push(true);
						bits.push(false);
						bits.push(true);
					}
					'E' | 'e' => {
						bits.push(true);
						bits.push(true);
						bits.push(true);
						bits.push(false);
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

			/// returns if the integer that the string would parse into is even
			fn string_is_even(str: &Vec<char>) -> bool {
				match str.last() {
					Some('0') | Some('2') | Some('4') | Some('6') | Some('8') => true,
					_ => false
				}
			}

			/// subtracts one from the integer represented by the string
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

			/// divides the integer represented by the string by 2
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
			
			// verify all characters are valid
			for c in stripped_arg.chars() {
				if !c.is_digit(10) {
					return Err(format!("Character {} not allowed in decimal numbers", c));
				};
			};
			
			// parse through input string
			let mut str: Vec<char> = stripped_arg.chars().collect();
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

	// trim leading zeroes\
	while !bits.is_empty() && !bits[0] {
		bits.remove(0);
	};

	// increase length of bits to write_length
	let target_len = match write_length {
		WriteLength::Unfixed => {
			let min_len = bits.len() as u64;
			let int = match write_mode {
				WriteMode::Hex(_) => 4u64,
				WriteMode::Octal => 3u64,
				WriteMode::Binary | WriteMode::Decimal => 1u64
			};
			min_len.next_multiple_of(int)
		}
		WriteLength::RoundUp => {
			let min_len = bits.len() as u64;
			let int = match write_mode {
				WriteMode::Binary | WriteMode::Hex(_) => 8u64,
				WriteMode::Octal => 6u64,
				WriteMode::Decimal => 1u64
			};
			min_len.next_multiple_of(int)
		}
		WriteLength::Fixed(len) => match write_mode {
			WriteMode::Decimal => bits.len() as u64,
			WriteMode::Octal => len * 6,
			WriteMode::Binary | WriteMode::Hex(_) => len * 8
		}
	};
	if (bits.len() as u64) > target_len {
		return Err("Number unrepresentable in fixed width".to_string());
	}
	while (bits.len() as u64) < target_len {
		bits.insert(0, false);
	}

	// flip bits and add one if reading from decimal and negative
	if negative_arg && read_mode == ReadMode::Decimal {
		negative(&mut bits);
	}
	
	return Ok(bits);
}

/// Converts the stream of bits representing a little-endian integer (signedness indicated by signed_mode) into
/// a string version of the integer in the format given by write_mode
fn write(mut bits: &mut BitVec, write_mode: WriteMode, write_separator: &WriteSeparator, signed_mode: bool, write_prefix: bool) -> String {
	let mut ret_str = if write_prefix {
		match write_mode {
			WriteMode::Binary => "0b",
			WriteMode::Octal => "0o",
			WriteMode::Hex(_) => "0x",
			WriteMode::Decimal => ""
		}.to_string()
	} else {
		String::new()
	};
	if bits.is_empty() {
		ret_str.push('0');
		return ret_str;
	};
	
	if let WriteMode::Decimal = write_mode {
		// do decimal conversion and return

		// handle negatives for decimal
		if signed_mode && write_mode == WriteMode::Decimal && bits.first().is_some_and(|b| *b) {
			ret_str.push('-');
			negative(&mut bits);
		};

		/// adds one to the integer represented by the string
		fn add_string(str: &mut Vec<char>) {
			// add to numbers
			for index in (0..str.len()).rev() {
				let ch = match str[index] {
					'9' => '0',
					'8' => '9',
					'7' => '8',
					'6' => '7',
					'5' => '6',
					'4' => '5',
					'3' => '4',
					'2' => '3',
					'1' => '2',
					'0' => '1',
					_ => panic!()
				};
				str[index] = ch;
				if ch != '0' { return; };
			};
			// leftover carry, add to front of str
			str.insert(0, '1');
		}

		/// multiplies the integer represented by the string by 2
		fn mult_string(str: &mut Vec<char>) {
			// multiply numbers starting in the back
			let mut carry = false;
			for index in (0..str.len()).rev() {
				str[index] = match str[index] {
					'9' => {
						if carry {
							'9'
						} else {
							carry = true;
							'8'
						}
					}
					'8' => {
						if carry {
							'7'
						} else {
							carry = true;
							'6'
						}
					}
					'7' => {
						if carry {
							'5'
						} else {
							carry = true;
							'4'
						}
					}
					'6' => {
						if carry {
							'3'
						} else {
							carry = true;
							'2'
						}
					}
					'5' => {
						if carry {
							'1'
						} else {
							carry = true;
							'0'
						}
					}
					'4' => {
						if carry {
							carry = false;
							'9'
						} else {
							'8'
						}
					}
					'3' => {
						if carry {
							carry = false;
							'7'
						} else {
							'6'
						}
					}
					'2' => {
						if carry {
							carry = false;
							'5'
						} else {
							'4'
						}
					}
					'1' => {
						if carry {
							carry = false;
							'3'
						} else {
							'2'
						}
					}
					'0' => {
						if carry {
							carry = false;
							'1'
						} else {
							'0'
						}
					}
					_ => panic!()
				};
			};
			
			// leftover carry, add to front of str
			if carry {
				str.insert(0, '1');
			}
		}

		// convert to vector of chars for greater mutability
		let mut num_str = Vec::new();
		for bit in bits {
			mult_string(&mut num_str);
			if *bit { add_string(&mut num_str); };
		}

		let mut chars_in_group = (3 - (num_str.len() % 3)) % 3;

		// push char vector into final string
		let mut iter = num_str.iter().peekable();
		while let Some(c) = iter.next() {
			ret_str.push(*c);

			chars_in_group += 1;
			if let WriteSeparator::Separator(sep) = write_separator {
				if chars_in_group == 3 && iter.peek().is_some() {
					ret_str.push_str(sep);
					chars_in_group = 0;
				};
			};
		}
		

		return ret_str;
	};

	let mut index_to_char = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
	let num_bits = match write_mode {
		WriteMode::Binary => 1,
		WriteMode::Octal => 3,
		WriteMode::Hex(is_upper) => {
			// fix index_to_char if lowercase
			if !is_upper {
				for i in 10..16 {
					index_to_char[i] = index_to_char[i].to_lowercase().next().unwrap();
				};
			};
			4
		}
		WriteMode::Decimal => panic!(),
	};

	let ideal_chars_in_group = match write_mode {
		WriteMode::Binary => 8,
		WriteMode::Octal | WriteMode::Hex(_) => 2,
		WriteMode::Decimal => panic!()
	};

	// number of chars already added to the group for emplacing separators
	let mut chars_in_group = (ideal_chars_in_group - (bits.len() / num_bits % ideal_chars_in_group)) % ideal_chars_in_group;

	let mut iter = bits.iter().peekable();
	'outer: loop {
		let mut index: usize = 0;
		for _ in 0..num_bits {
			index <<= 1;
			if let Some(bit) = iter.next() {
				index |= *bit as usize;
			} else {
				// no more bits
				break 'outer;
			};
		};
		ret_str.push(index_to_char[index]);

		chars_in_group += 1;
		if let WriteSeparator::Separator(sep) = write_separator {
			if chars_in_group == ideal_chars_in_group && iter.peek().is_some() {
				ret_str.push_str(sep);
				chars_in_group = 0;
			};
		};
	};

	return ret_str;
}

/// Converts the given argument into the specified format and returns either the converted string or an error message
fn convert(ref arg: &String, read_mode: ReadMode, write_mode: WriteMode, write_length: WriteLength, write_separator: &mut WriteSeparator, signed_mode: bool, write_prefix: bool) -> Result<String, String> {
	// runtime fix write_separator
	if let WriteSeparator::RuntimeDetermine = write_separator {
		*write_separator = WriteSeparator::Separator(match write_mode {
			WriteMode::Decimal => ',',
			WriteMode::Binary | WriteMode::Octal | WriteMode::Hex(_) => ' '
		}.to_string());
	}
	
	// do conversion
	match read(arg, read_mode, write_mode, write_length, signed_mode) {
		Ok(mut bits) => {
			Ok(write(&mut bits, write_mode, &write_separator, signed_mode, write_prefix))
		}
		Err(msg) => {
			Err(msg)
		}
	}
}

/// Converts numbers into different representations
fn main() {
	// get args in a queue, use queue capacity to trim first arg
	let mut args = CircularBuffer::new(std::env::args().len() - 1);
	for arg in std::env::args() {
		let _ = args.add(arg);
	}

	// set standard settings
	let mut read_mode = ReadMode::Interpret;
	let mut write_mode = WriteMode::Hex(true);
	let mut write_length = WriteLength::Unfixed;
	let mut write_separator = WriteSeparator::None;
	let mut signed_mode = false;
	let mut write_prefix = true;

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
			['-', 'x'] | ['-', 'x', 'u'] => {
				write_mode = WriteMode::Hex(true);
			}
			['-', 'x', 'l'] => {
				write_mode = WriteMode::Hex(false);
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
				write_separator = WriteSeparator::RuntimeDetermine;
			}
			['-', 't'] => {
				write_separator = WriteSeparator::None;
			}
			
			['-', 'p'] => {
				write_prefix = true;
			}
			
			['-', 'n'] => {
				write_prefix = false;
			}
			['-', 'v'] | ['-', 'V'] => {
				println!("Hex v{}", env!("CARGO_PKG_VERSION"));
				exit(0);
			}
			_ => {
				// something else (assume number)
				match convert(&arg, read_mode, write_mode, write_length, &mut write_separator, signed_mode, write_prefix) {
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
				line = line.trim().to_string();
				match convert(&line, read_mode, write_mode, write_length, &mut write_separator, signed_mode, write_prefix) {
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

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    use crate::*;

	fn bitvec_of_num(mut num: u64) -> BitVec {
		let mut bv: BitVec = BitVec::new();
		while num != 0 {
			bv.push((num & 1) == 1);
			num >>= 1;
		};
		bv.reverse();
		bv
	}

	fn padded_bitvec_of_num(num: u64, pad: usize) -> BitVec {
		let mut bv = bitvec_of_num(num);
		while bv.len() % pad != 0 {
			bv.insert(0, false);
		}
		bv
	}

	#[test]
	fn bitvec_of_num_test() {
		let mut bv: BitVec = BitVec::new();
		bv.push(true);
		bv.push(false);
		bv.push(true);
		bv.push(true);
		assert_eq!(bv, bitvec_of_num(11));
		bv.insert(0, false);
		bv.insert(0, false);
		bv.insert(0, false);
		bv.insert(0, true);
		assert_eq!(bv, bitvec_of_num(139))
	}

	#[test]
	fn read_decimal_tests() {
		assert_eq!(read(&"0".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(0)));
		assert_eq!(read(&"1".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(1)));
		assert_eq!(read(&"2".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(2)));
		assert_eq!(read(&"34".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(34)));
		assert_eq!(read(&"65".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(65)));
		assert_eq!(read(&"789".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(789)));
	}

	#[test]
	fn read_negative_decimal_tests() {
		assert_eq!(read(&"-0".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Unfixed, true), Ok(bitvec_of_num(0)));
		assert_eq!(read(&"-1".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(8), true), Ok(bitvec_of_num(-1i64 as u64)));
		assert_eq!(read(&"-2".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(8), true), Ok(bitvec_of_num(-2i64 as u64)));
		assert_eq!(read(&"-34".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(8), true), Ok(bitvec_of_num(-34i64 as u64)));
		assert_eq!(read(&"-65".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(8), true), Ok(bitvec_of_num(-65i64 as u64)));
		assert_eq!(read(&"-789".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(8), true), Ok(bitvec_of_num(-789i64 as u64)));
	}

	#[test]
	fn read_octal_tests() {
		assert_eq!(read(&"0".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(0)));
		assert_eq!(read(&"1".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(1)));
		assert_eq!(read(&"2".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(2)));
		assert_eq!(read(&"34".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(28)));
		assert_eq!(read(&"65".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(53)));
		assert_eq!(read(&"7770".to_string(), ReadMode::Octal, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(4088)));
	}

	#[test]
	fn read_binary_tests() {
		assert_eq!(read(&"0".to_string(), ReadMode::Binary, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(0)));
		assert_eq!(read(&"1".to_string(), ReadMode::Binary, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(1)));
		assert_eq!(read(&"1010".to_string(), ReadMode::Binary, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(10)));
		assert_eq!(read(&"1111111111".to_string(), ReadMode::Binary, WriteMode::Decimal, WriteLength::Unfixed, false), Ok(bitvec_of_num(1023)));
	}

	#[test]
	fn write_decimal_tests() {
		assert_eq!(write(&mut bitvec_of_num(0), WriteMode::Decimal, &WriteSeparator::None, false, false), "0".to_string());
		assert_eq!(write(&mut bitvec_of_num(1), WriteMode::Decimal, &WriteSeparator::None, false, false), "1".to_string());
		assert_eq!(write(&mut bitvec_of_num(2), WriteMode::Decimal, &WriteSeparator::None, false, false), "2".to_string());
		assert_eq!(write(&mut bitvec_of_num(34), WriteMode::Decimal, &WriteSeparator::None, false, false), "34".to_string());
		assert_eq!(write(&mut bitvec_of_num(65), WriteMode::Decimal, &WriteSeparator::None, false, false), "65".to_string());
		assert_eq!(write(&mut bitvec_of_num(789), WriteMode::Decimal, &WriteSeparator::None, false, false), "789".to_string());
		
		assert_eq!(write(&mut BitVec::new(), WriteMode::Decimal, &WriteSeparator::None, false, true), "0".to_string());
		assert_eq!(write(&mut bitvec_of_num(2024), WriteMode::Decimal, &WriteSeparator::None, false, true), "2024".to_string());
	}
	
	#[test]
	fn write_negative_decimal_tests() {
		assert_eq!(write(&mut bitvec_of_num(0), WriteMode::Decimal, &WriteSeparator::None, true, false), "0".to_string());
		assert_eq!(write(&mut bitvec_of_num(-1i64 as u64), WriteMode::Decimal, &WriteSeparator::None, true, false), "-1".to_string());
		assert_eq!(write(&mut bitvec_of_num(-2i64 as u64), WriteMode::Decimal, &WriteSeparator::None, true, false), "-2".to_string());
		assert_eq!(write(&mut bitvec_of_num(-34i64 as u64), WriteMode::Decimal, &WriteSeparator::None, true, false), "-34".to_string());
		assert_eq!(write(&mut bitvec_of_num(-65i64 as u64), WriteMode::Decimal, &WriteSeparator::None, true, false), "-65".to_string());
		assert_eq!(write(&mut bitvec_of_num(-789i64 as u64), WriteMode::Decimal, &WriteSeparator::None, true, false), "-789".to_string());
	}

	#[test]
	fn write_octal_tests() {
		assert_eq!(write(&mut padded_bitvec_of_num(0, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "0".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(1, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "1".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(2, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "2".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(28, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "34".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(53, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "65".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(4088, 3), WriteMode::Octal, &WriteSeparator::None, false, false), "7770".to_string());
		
		assert_eq!(write(&mut BitVec::new(), WriteMode::Octal, &WriteSeparator::None, false, true), "0o0".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(2024, 3), WriteMode::Octal, &WriteSeparator::None, false, true), "0o3750".to_string());
	}

	#[test]
	fn write_hex_tests() {
		assert_eq!(write(&mut padded_bitvec_of_num(0, 4), WriteMode::Hex(false), &WriteSeparator::None, false, false), "0".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(1, 4), WriteMode::Hex(false), &WriteSeparator::None, false, false), "1".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(2, 4), WriteMode::Hex(false), &WriteSeparator::None, false, false), "2".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(52, 4), WriteMode::Hex(false), &WriteSeparator::None, false, false), "34".to_string());
		
		assert_eq!(write(&mut BitVec::new(), WriteMode::Hex(false), &WriteSeparator::None, false, true), "0x0".to_string());
		assert_eq!(write(&mut padded_bitvec_of_num(2024, 4), WriteMode::Hex(false), &WriteSeparator::None, false, true), "0x7e8".to_string());
	}

	#[test]
	fn write_binary_tests() {
		assert_eq!(write(&mut bitvec_of_num(0), WriteMode::Binary, &WriteSeparator::None, false, false), "0".to_string());
		assert_eq!(write(&mut bitvec_of_num(1), WriteMode::Binary, &WriteSeparator::None, false, false), "1".to_string());
		assert_eq!(write(&mut bitvec_of_num(2), WriteMode::Binary, &WriteSeparator::None, false, false), "10".to_string());
		assert_eq!(write(&mut bitvec_of_num(52), WriteMode::Binary, &WriteSeparator::None, false, false), "110100".to_string());
		
		assert_eq!(write(&mut BitVec::new(), WriteMode::Binary, &WriteSeparator::None, false, true), "0b0".to_string());
		assert_eq!(write(&mut bitvec_of_num(2024), WriteMode::Binary, &WriteSeparator::None, false, true), "0b11111101000".to_string());
	}

	#[test]
	fn hex_capitalization_tests() {
		assert_eq!(convert(&"abcdef".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("abcdef".to_string()));
		assert_eq!(convert(&"ABCDEF".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("abcdef".to_string()));
		assert_eq!(convert(&"abcdef".to_string(), ReadMode::Hex, WriteMode::Hex(true), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("ABCDEF".to_string()));
		assert_eq!(convert(&"ABCDEF".to_string(), ReadMode::Hex, WriteMode::Hex(true), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("ABCDEF".to_string()));
	}

	#[test]
	fn interpret_type_tests() {
		assert_eq!(convert(&"abcdef".to_string(), ReadMode::Interpret, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("abcdef".to_string()));
		assert_eq!(convert(&"0xabcdef".to_string(), ReadMode::Interpret, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("abcdef".to_string()));
		assert_eq!(convert(&"0b1010".to_string(), ReadMode::Interpret, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("a".to_string()));
		assert_eq!(convert(&"255".to_string(), ReadMode::Interpret, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("ff".to_string()));
		assert_eq!(convert(&"0o377".to_string(), ReadMode::Interpret, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("ff".to_string()));
	}

	#[test]
	fn fixed_width_tests() {
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Fixed(2), &mut WriteSeparator::None, false, false), Ok("5".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(1), &mut WriteSeparator::None, false, false), Ok("00000101".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::Fixed(1), &mut WriteSeparator::None, false, false), Ok("05".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::Fixed(1), &mut WriteSeparator::None, false, false), Ok("05".to_string()));
		assert_eq!(convert(&"15".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Fixed(2), &mut WriteSeparator::None, false, false), Ok("0000000000001111".to_string()));
		assert_eq!(convert(&"256".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::Fixed(3), &mut WriteSeparator::None, false, false), Ok("000100".to_string()));
		assert_eq!(convert(&"64".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::Fixed(3), &mut WriteSeparator::None, false, false), Ok("000100".to_string()));
		
		match convert(&"fff".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Fixed(1), &mut WriteSeparator::None, false, false) {
			Err(_) => { }
			Ok(_) => panic!()
		}
	}

	#[test]
	fn rounded_width_tests() {
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("5".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("00000101".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("05".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("05".to_string()));
		assert_eq!(convert(&"256".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("0000000100000000".to_string()));
		assert_eq!(convert(&"256".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("0100".to_string()));
		assert_eq!(convert(&"64".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::RoundUp, &mut WriteSeparator::None, false, false), Ok("0100".to_string()));
	}

	#[test]
	fn unfixed_width_tests() {
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("5".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("101".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("5".to_string()));
		assert_eq!(convert(&"5".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("5".to_string()));
		assert_eq!(convert(&"256".to_string(), ReadMode::Decimal, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("100000000".to_string()));
		assert_eq!(convert(&"256".to_string(), ReadMode::Decimal, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("100".to_string()));
		assert_eq!(convert(&"64".to_string(), ReadMode::Decimal, WriteMode::Octal, WriteLength::Unfixed, &mut WriteSeparator::None, false, false), Ok("100".to_string()));
	}

	#[test]
	fn separator_tests() {
		assert_eq!(convert(&"5000".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("5,000".to_string()));
		assert_eq!(convert(&"50000".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("50,000".to_string()));
		assert_eq!(convert(&"500000".to_string(), ReadMode::Decimal, WriteMode::Decimal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("500,000".to_string()));
		assert_eq!(convert(&"bcd".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("b cd".to_string()));
		assert_eq!(convert(&"abcd".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("ab cd".to_string()));
		assert_eq!(convert(&"12".to_string(), ReadMode::Octal, WriteMode::Octal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("12".to_string()));
		assert_eq!(convert(&"123".to_string(), ReadMode::Octal, WriteMode::Octal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("1 23".to_string()));
		assert_eq!(convert(&"1234".to_string(), ReadMode::Octal, WriteMode::Octal, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("12 34".to_string()));
		assert_eq!(convert(&"10101010".to_string(), ReadMode::Binary, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("10101010".to_string()));
		assert_eq!(convert(&"1010101010".to_string(), ReadMode::Binary, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("10 10101010".to_string()));
		assert_eq!(convert(&"10101010101010".to_string(), ReadMode::Binary, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("101010 10101010".to_string()));
		assert_eq!(convert(&"1010101010101010".to_string(), ReadMode::Binary, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("10101010 10101010".to_string()));
		assert_eq!(convert(&"101010101010101010".to_string(), ReadMode::Binary, WriteMode::Binary, WriteLength::Unfixed, &mut WriteSeparator::RuntimeDetermine, false, false), Ok("10 10101010 10101010".to_string()));
	
		assert_eq!(convert(&"abcd".to_string(), ReadMode::Hex, WriteMode::Hex(false), WriteLength::Unfixed, &mut WriteSeparator::Separator("hey".to_string()), false, false), Ok("abheycd".to_string()));
	}
}