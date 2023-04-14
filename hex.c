#include <stdio.h>

#define VERSION "Version 0.9"

#define PRINT_WARNINGS !(options & 0x20)
#define FIXED_WIDTH (options & 0x08)
#define FORCE_READ_MODE (options & 0x40)
#define SPACE_BYTES (options & 0x80)
#define SIGNED (options & 0x10)
#define TC_MIN 0x8000000000000000

enum mode{
    binary = 1,
    decimal = 0,
    hex = 4,
    octal = 3
};

typedef enum mode mode;

void print_help(){
    printf("HEX\n");
    printf("Tool for converting between different number types\n");
    printf("Usage: hex <options> <params>\n");
    printf("Can take many params at once or leave empty to read from stdin\n");
    printf("\n");
    printf("Options:\n");
    printf("-? Displays this help\n");
    printf("-c Forces the program to read input as a a binary\n");
    printf("-n Forces the program to read input as a base 10 (decimal) integer\n");
    printf("-e Forces the program to read input as an octal\n");
    printf("-q Forces the program to read input as a hexadecimal\n");
    printf("-f Lets the program decide how to read input based off prefix (default)\n");
    printf("-b Writes output in binary with prefix\n");
    printf("-B Writes output in binray, broken into blocks of 8\n");
    printf("-d Writes output in base 10 (decimal)\n");
    printf("-o Writes output in octal with prefix\n");
    printf("-h Writes output in hexadecimal with prefix\n");
    printf("-wN Sets the width of output to be N bytes long.  N must be 1-8 (inclusive).\n");
    printf("       Not intended for use with octal or decimal\n");
    printf("-z Suppresses warning messages\n");
    printf("-v Prints the program version and exits\n");
    printf("\n");
}

char value_of_char(unsigned char c, mode readMode){
    c -= 48;
    if(c & 0x80) return -1;
    if(c < 2) return c;
    if(readMode == binary) return -1;
    if(c < 8) return c;
    if(readMode == octal) return -1;
    if(c < 10) return c;
    if(readMode == decimal) return -1;
    c -= 17;
    if(c < 6) return 10 + c;
    c -= 32;
    if(c < 6) return 10 + c;
    return -1;
}

long long read_num_decimal(char* string, char options){
    char* str = string;
    unsigned long long out = 0;
    unsigned long long temp;
    while(*str){
	char value = value_of_char(*str, decimal);
	if(~value){
	    temp = out;
	    out *= 10;
	    out += value;
	    if(temp > out) fprintf(stderr, "Decimal reading overflow\n");
	}
	str++;
    }
    if(SIGNED){
	//exception if 10000... and negative, then does not overflow, gets set to TC_MIN
	//clear top bit (set 0) and check for TC overflow
	//check for negatives, flip and add one
	char isNeg = *string == '_';
	if(out == TC_MIN && isNeg) return out;
	if(out & TC_MIN){//top bit is a one
	    if(!isNeg || out ^ TC_MIN){//out is greater than TC_MIN, overflow will occur
		out = out ^ TC_MIN;//flip top bit to 0
		if(PRINT_WARNINGS) fprintf(stderr, "TC reading overflow\n");
	    }
	    else return out;//out is TC_MIN
	}
	//if negative, flip and add one
	if(isNeg){
	    out = ~out;
	    out++;
	}
    }
    return out;
}

long long read_num(char* string, mode readMode, char options){
    if(readMode == decimal) return read_num_decimal(string, options);
    long long out = 0;
    int total_bits = 0;
    char value;
    while(*string){
	value = value_of_char(*string, readMode);
	if(~value){
	    out <<= (char)readMode;
	    out |= value;
	    total_bits += (char)readMode;
	}
	string++;
    }
    if(PRINT_WARNINGS && (total_bits > 64)){
	fprintf(stderr, "Number reading overflow by %i bits\n", total_bits - 64);
    }
    return out;
}

unsigned short get_sig_bits(long long num, char isSigned){
    num = num ^ ((long long)(num & TC_MIN) >> 63);
    unsigned short ret = 0;
    while(num){
       	num = ((unsigned long long)num) >> 1;
	ret++;
    }
    if(isSigned) ret++;
    return ret;
}

void print_num(long long num, mode outMode, unsigned char options){
    //determine number of bits to print
    unsigned short sig_bits = 0;
    if(!FIXED_WIDTH || PRINT_WARNINGS) sig_bits = get_sig_bits(num, SIGNED);
    if(FIXED_WIDTH){
	unsigned short fix_bits = ((options & 0x7) + 1) << 3;//8 * num bytes
	if(fix_bits < sig_bits) fprintf(stderr, "Fixed width results in loss of data\n");//printwarnings must be turned on for statement to evaluate true
	sig_bits = fix_bits;
    }
    if(!sig_bits) sig_bits = 1;
    //cut/sign extend extra bits
    unsigned char shift = 64 - sig_bits;
    num <<= shift;
    if(SIGNED) num >>= shift;
    else num = ((unsigned long long)num) >> shift;
    //print decimal
    if(outMode == decimal){
	if(SIGNED) printf("%lli\n", num);
	else printf("%llu\n", num);
	return;
    }
    //round up sig_bits
    if(outMode == hex || SPACE_BYTES){
	if(sig_bits & 0x1) sig_bits++;
	if(sig_bits & 0x2) sig_bits += 2;
	if(SPACE_BYTES && (sig_bits & 0x4)) sig_bits += 4;
    }
    else if(outMode == octal && sig_bits / 3) sig_bits += (3 - (sig_bits % 3));
    //print prefix
    switch(outMode){
	case hex:
	    printf("0x");
	    break;
	case octal:
	    printf("0");
	    break;
	case binary:
	    if(!SPACE_BYTES) printf("0b");
	    break;
    }
    //create mask
    char mask = 0x1;
    for(unsigned char i = 1; i < outMode; i++) mask |= (mask << 1);
    //print pace separated binary
    if(SPACE_BYTES){
	char counter = 0;
	do{
	    if(counter == 8){
		counter = 0;
		printf(" ");
	    }
	    sig_bits--;
	    printf("%i", mask & (num >> sig_bits));
	    counter++;
	}while(sig_bits);
	printf("\n");
	return;
    }
    //print all
    do{
	sig_bits -= outMode;
	printf("%X", mask & (num >> sig_bits));
    }while(sig_bits);
    printf("\n");
}

void num_parse(char* string, mode readMode, mode outMode, unsigned char options){
    if(!FORCE_READ_MODE){
	//infer readMode
	if(*string != '0') readMode = decimal;
	else if(string[1] == 'x') readMode = hex;
	else if(string[1] == 'b') readMode = binary;
	else readMode = octal;
    }
    print_num(read_num(string, readMode, options), outMode, options);
}

int main(int argc, char** argv){
   argc--;
   argv++;
   mode readMode = decimal;
   mode outMode = hex;
   unsigned char options = 0x0;
   /*
    * The char options stores multiple settings in each of its bits
    * The bits are laid out as:
    * MSB           LSB
    *  7 6 5 4 3 2 1 0
    * Where:
    * 7 is a boolean value for if spaces should be put between each byte (in binary printing)
    * 6 is a boolean value for if the program is forced to use a certain type of number for input reading
    * 5 is a boolean value for if the program should suppress error messages
    * 4 is a boolean value representing if the number is unsigned (as opposed to two's complement signed)
    * 3 is a boolean value for if the number of bits/bytes that should be printed is fixed
    * 2-0 is a 3 bit integer representing one less than the number of bytes that should be printed, assuming 3 is true
    */
   unsigned char converted = 0;
   char** args_end = argv + argc;
   //arg loop
   while(argv != args_end){
       char* arg = *argv;
       //options/flag characters
       if(*arg == '-' || *arg == '/'){
	   arg++;
	   while(*arg){
	       switch(*arg){
		   //set options based off character
		   case '?'://print help
		       print_help();
		       goto end;
		   case 'c'://read binary
		       readMode = binary;
		       options |= 0x40;
		       break;
		   case 'n'://read decimal
		       readMode = decimal;
		       options |= 0x40;
		       break;
		   case 'e'://read octal
		       readMode = octal;
		       options |= 0x40;
		       break;
		   case 'q'://read hex
		       readMode = hex;
		       options |= 0x40;
		       break;
		   case 'f'://program autodetermines read mode from prefix (default)
		       options &= 0xBF;
		   case 'b'://output in binary, no spaces
		       outMode = binary;
		       options &= 0x7F;
		       break;
		   case 'd'://output in decimal
		       outMode = decimal;
		       options &= 0x7F;
		       break;
		   case 'o'://output in octal
		       outMode = octal;
		       options &= 0x7F;
		       break;
		   case 'h'://output in hex (default)
		       outMode = hex;
		       options &= 0x7F;
		       break;
		   case 'B'://output in binary, split into bytes
		       outMode = binary;
		       options |= 0x80;
		       break;
		   case 't'://two's complement
		       options |= 0x10;
		       break;
		   case 'w'://fixed-width output
		       char i = 0;
		       for(i = 0; i < 2; i++){
			   arg++;
		           unsigned char a = (unsigned char)(*arg - 49);
			   if(a < 8){
			      options |= (a & (char)0x07);
			      options |= 0x08;
			      goto found_param;
			   }
		       }
		       arg -= i;
		       if(PRINT_WARNINGS) fprintf(stderr, "Failed to find valid parameter for option 'w'\n");
           found_param:break;
		   case 'z'://suppress warnings
		       options |= 0x20;
		       break;
		   case 'v'://print version
		       printf("%s\n", VERSION);
		       goto end;
		       break;
		   default:
		       if(PRINT_WARNINGS) fprintf(stderr, "Unknown flag: %c\n", *arg);
		       break;
	       }
	       arg++;
	   }
       }
       //end options
       else{//convert
	   num_parse(arg, readMode, outMode, options);
	   converted++;
       }
       argv++;
   }
   //start reading stdin if no conversion paramters
   if(!converted){
       char* line = 0x0;
       size_t len = 0;
       while(1){
	   getline(&line, &len, stdin);
	   if(*line == '\n') goto end;
	   num_parse(line, readMode, outMode, options);
	   converted++;
       }
   }
end: return converted;
}
