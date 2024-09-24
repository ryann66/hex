# Hex
Tool for converting between different number types
Usage: hex <options> <params>
Can take many params at once or be left empty to read from stdin

## Options:
-h Displays this help and exits
-v Displays the program version and exits
-B Forces the program to read input as a a binary
-D Forces the program to read input as a base 10 (decimal) integer
-O Forces the program to read input as an octal
-X Forces the program to read input as a hexadecimal
-F Lets the program decide how to read input based off prefix (default)
-b Writes output in binary with prefix
-d Writes output in base 10 (decimal)
-o Writes output in octal with prefix
-x Writes output in hexadecimal with prefix
        Default is to print uppercase hex; use -xl to force lowercase
-s Puts the system into signed mode (two's complement).  Use '-' in decimals
-u Puts the system into unsigned mode (default)
-w=<Num> Sets the length of output in bytes
        When writing in octal uses a 6-bit byte. Has no effect when writing in decimal
-f Sets the width of ouput to the minimum number of characters to represent the number
-r Rounds the width of output to a pretty length (usually a byte boundary)
        Octal and binary will be rounded to bytes, octal will be rounded to even lengths
        This option does not effect the print length of decimal numbers
-c[=<sep>] Adds a separator character between groups of digits
        Separator is added every 3 chars for decimal, 2 for hex, 8 for binary, and 2 for octal
        Default separator is ',' for decimal and ' ' for everything else
-t Removes the separator character
-p Write prefixes on all non-decimal numbers (default)
-n Omit prefixes from all numbers