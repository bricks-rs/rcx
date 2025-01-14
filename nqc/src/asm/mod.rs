pub mod ast;
pub mod parser;

// 1. lexer to take source asm file into Vec<LabelOrOpcode>
//    - parse hex and decimal numbers
// 2. parser to convert string opcode names into proper opcode types
//    and validate they they have the correct arguments
// 3. assembler to convert into bytecode and resolve labels & jumps
// 4. something to write out the binary file
