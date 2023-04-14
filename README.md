# hex
Command line converter for Linux

Usage:
hex \<options\> \<params\>  
Params are the numbers to be converted.  Use prefixes or options to interpret non-decimal numbers.  
If no params are passed then program will read from stdin (send an empty line to stop reading)  
Program returns the number of numbers converted upon completion  

  
**Options**  
-? Displays help  
-c Forces the program to read input as a a binary  
-n Forces the program to read input as a base 10 (decimal) integer  
-e Forces the program to read input as an octal  
-q Forces the program to read input as a hexadecimal  
-f Lets the program decide how to read input based off prefix (default)  
-b Writes output in binary with prefix  
-B Writes output in binary, broken into blocks of 8  
-d Writes output in base 10 (decimal)  
-o Writes output in octal with prefix  
-h Writes output in hexadecimal with prefix  
-t Puts the system into signed mode (two's complement).  
       Use '_' for '-' when writing decimals to avoid confusion with flags  
-w**N** Sets the width of output to be **N** bytes long.  **N** must be 1-8 (inclusive).  
       Not intended for use with octal or decimal  
-z Suppresses warning messages  
-v Prints the program version and exits  
