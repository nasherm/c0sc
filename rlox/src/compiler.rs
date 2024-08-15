use std::ops::Deref;
use crate::vm::InterpretResult;
use crate::scanner::{Scanner, TokenType, Token};
use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use std::{rc::Rc, cell::RefCell};
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::ops::Add;

struct Parser {
    current: Token, 
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, FromPrimitive)]
#[allow(non_camel_case_types)]
enum Precedence {
    PREC_NONE = 0,
    PREC_ASSIGNMENT,  // =
    PREC_OR,          // or
    PREC_AND,         // and
    PREC_EQUALITY,    // == !=
    PREC_COMPARISON,  // < > <= >=
    PREC_TERM,        // + -
    PREC_FACTOR,      // * /
    PREC_UNARY,       // ! -
    PREC_CALL,        // . ()
    PREC_PRIMARY
}

pub struct Compiler {
    scanner: Scanner,
    parser: Parser, 
    compiling_chunk: Rc<RefCell<Chunk>>,
}

type ParseFn = fn(&mut Compiler) -> ();
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    prec: Precedence,
}

lazy_static! {
    static ref PARSE_RULES: HashMap<TokenType, ParseRule> = Compiler::init_parse_rules();
}

impl Compiler {
    pub fn init(source_code: &String) -> Self {
        Compiler {
            scanner: Scanner::init(&source_code),
            parser: Parser {
                current: Token::init(), 
                previous: Token::init(),
                had_error: false,
                panic_mode: false,
            },
            compiling_chunk: Rc::new(RefCell::new(Chunk::init())),
        }
    }

    fn init_parse_rules() -> HashMap<TokenType, ParseRule> {
        HashMap::from([
            (TokenType::TOKEN_LEFT_PAREN, ParseRule { prefix: Some(Compiler::grouping), infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_RIGHT_PAREN, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_LEFT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_RIGHT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_COMMA, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_DOT, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_MINUS, ParseRule { prefix: Some(Compiler::unary), infix: Some(Compiler::binary), prec: Precedence::PREC_TERM }),
            (TokenType::TOKEN_PLUS, ParseRule { prefix: None, infix: Some(Compiler::binary), prec: Precedence::PREC_TERM }),
            (TokenType::TOKEN_SEMICOLON, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_SLASH, ParseRule { prefix: None, infix: Some(Compiler::binary), prec: Precedence::PREC_FACTOR }),
            (TokenType::TOKEN_STAR, ParseRule { prefix: None, infix: Some(Compiler::binary), prec: Precedence::PREC_FACTOR }),
            (TokenType::TOKEN_BANG, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_BANG_EQUAL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_EQUAL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_EQUAL_EQUAL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_GREATER, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_GREATER_EQUAL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_LESS, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_LESS_EQUAL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_IDENTIFIER, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_STRING, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_NUMBER, ParseRule { prefix: Some(Compiler::number), infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_AND, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_CLASS, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_ELSE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_FALSE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_FOR, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_FUN, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_IF, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_NIL, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_OR, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_PRINT, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_RETURN, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_SUPER, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_THIS, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_TRUE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_VAR, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_WHILE, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_ERROR, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
            (TokenType::TOKEN_EOF, ParseRule { prefix: None, infix: None, prec: Precedence::PREC_NONE }),
        ])
    }


    pub fn interpret(&mut self) -> InterpretResult {
        let mut line = -1;
        loop {
            let token = self.scanner.scan_token();
            if token.line != line {
                print!("{}", token.line);
                line = token.line;
            } else {
                print!("    | ");
            }
            let token_chars = self.scanner.token_chars(token.length);
            print!("{:?}  {:?}", token.token_type, token_chars);
            if token.token_type == TokenType::TOKEN_EOF {
                break
            }
        }
        InterpretResult::InterpretOk
    }

    pub fn compile(&mut self) -> bool {
        self.compiling_chunk = Rc::new(RefCell::new(Chunk::init()));
        self.advance();
        self.expression();
        self.consume(TokenType::TOKEN_EOF, "Expect end of expression.");
        self.end_compiler();
        self.parser.had_error
    }

    pub fn current_chunk(&self) -> Result<RefCell<Chunk>, Rc<RefCell<Chunk>>> {
        Rc::try_unwrap(self.compiling_chunk.to_owned())
    }

    fn end_compiler(&mut self) -> () {
        self.emit_return();
    }

    fn unary(&mut self) -> () {
        let op_type = self.parser.previous.token_type;
        self.parse_precedence(Precedence::PREC_UNARY);
        match op_type {
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OpNegate),
            _ => (),
        }
    }

    fn get_rule(&self, token_type: TokenType) -> Option<&ParseRule> {
        PARSE_RULES.get(&token_type)
    }

    fn binary(&mut self) -> () {
        let op_type = self.parser.previous.token_type;
        let parse_rule = self.get_rule(op_type);
        match parse_rule {
            Some(rule) => {
                let prec = rule.prec as u32+ 1;
                self.parse_precedence(num::FromPrimitive::from_u32(prec).unwrap())
            },
            _ => (),
        };

        match op_type {
            TokenType::TOKEN_PLUS => self.emit_byte(OpCode::OpAdd),
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OpSub),
            TokenType::TOKEN_STAR => self.emit_byte(OpCode::OpMult),
            TokenType::TOKEN_SLASH => self.emit_byte(OpCode::OpDiv),
            _ => panic!(), // Unreachable
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> () {
        self.advance();
        let prefix_rule = self.get_rule(self.parser.previous.token_type).unwrap().prefix;
        match prefix_rule {
            Some(f) => f(self),
            _ => panic!("Expected expression"),
        }

        while precedence <= self.get_rule(self.parser.current.token_type).unwrap().prec {
            self.advance();
            match self.get_rule(self.parser.previous.token_type).unwrap().infix {
                Some(f) => f(self),
                _ => panic!("Expected infix rule"),
            }
        }
    }

    fn grouping(&mut self) -> () {
        self.expression();
        self.consume(TokenType::TOKEN_RIGHT_PAREN, "Expect ')' after expression");
    }

    fn number(&mut self) -> () {
        let value = match &self.parser.previous.option_string {
            Some(num_string) => num_string.parse::<f64>().unwrap(),
            _ => panic!(),
        };
        self.emit_constant(Value::num_val(value));
    }

    fn emit_constant(&mut self, value: Value) -> () {
        let op_constant_index= self.make_constant(value);
        self.emit_bytes(OpCode::OpConstant, OpCode::Byte(op_constant_index));
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.compiling_chunk.borrow_mut().add_constant(value) as u8
    }

    fn emit_return(&mut self) -> () {
        self.emit_byte(OpCode::OpReturn);
    }

    fn error_at_current(&mut self, message: &str) -> () {
        self.error_at(self.parser.current.clone(), message);
    } 

    fn error(&mut self, message: &str) -> () {
        self.error_at(self.parser.previous.clone(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) -> () {
        if self.parser.panic_mode { // Error suppression
            return
        }

        self.parser.panic_mode = true;
        eprint!("[line {} Error]", token.line);
        match token.token_type {
            TokenType::TOKEN_EOF => eprint!(" at end"),
            TokenType::TOKEN_ERROR => (),
            _ => eprint!(" at {} {}", token.length, token.start),
        };
        eprint!(": {}", message);
        self.parser.had_error = true;
    }

    fn advance(&mut self) -> () {
        self.parser.previous = self.parser.current.clone();
        println!("{:?}", self.parser.previous);
        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.token_type != TokenType::TOKEN_ERROR {
                break
            }
            self.error_at_current("");
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> () {
        if self.parser.current.token_type == token_type {
            self.advance();
        }
        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: OpCode) -> () {
        self.compiling_chunk.borrow_mut().write_chunk(byte, self.parser.previous.line as u32)
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: OpCode) -> () {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn compare_chunk(&self, expected_code: &Vec<OpCode>, expected_value_array: &[f64]) -> () {
        self.compiling_chunk.borrow().compare_code(expected_code, expected_value_array);
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::OpCode;
    use crate::compiler::Compiler;

    fn test_compiler_init_and_compile(source_code: String) -> Compiler {
        let mut compiler = Compiler::init(&source_code);
        let compile_result = compiler.compile();
        if !compile_result {
            panic!("Failed to parse this program: {:?}", source_code);
        }
        compiler
    }

    #[test]
    fn parse_unary() {
        let compiler = test_compiler_init_and_compile(String::from("-1"));
        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpNegate,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;1] = [1.0];
        compiler.compare_chunk(&expected_code, &expected_value_array);
    }

    #[test]
    fn parse_binary_add() {
        let compiler = test_compiler_init_and_compile(String::from("1+2"));

        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpAdd,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;2] = [1.0, 2.0];
        compiler.compare_chunk(&expected_code, &expected_value_array)
    }

    #[test]
    fn parse_binary_sub() {
        let compiler = test_compiler_init_and_compile(String::from("3-4"));

        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpSub,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;2] = [3.0, 4.0];
        compiler.compare_chunk(&expected_code, &expected_value_array)
    }

    #[test]
    fn parse_binary_mult() {
        let compiler = test_compiler_init_and_compile(String::from("5*6"));

        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpMult,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;2] = [5.0, 6.0];
        compiler.compare_chunk(&expected_code, &expected_value_array)
    }

    #[test]
    fn parse_binary_div() {
        let compiler = test_compiler_init_and_compile(String::from("7 / 8"));
        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpDiv,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;2] = [7.0, 8.0];
        compiler.compare_chunk(&expected_code, &expected_value_array)
    }

    #[test]
    fn parse_binary_nested() {
        let compiler = test_compiler_init_and_compile(String::from("1 + (2 - (3 * (4 / 5) ) )"));
        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpConstant,
            OpCode::Byte(2),
            OpCode::OpConstant,
            OpCode::Byte(3),
            OpCode::OpConstant,
            OpCode::Byte(4),
            OpCode::OpDiv,
            OpCode::OpMult,
            OpCode::OpSub,
            OpCode::OpAdd,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64;5] = [1.0, 2.0, 3.0, 4.0, 5.0];
        compiler.compare_chunk(&expected_code, &expected_value_array)
    }

    #[test]
    fn parse_nested_variant_one() {
        let compiler = test_compiler_init_and_compile(String::from("1 + (2 - 3)  * 4"));
        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpConstant,
            OpCode::Byte(2),
            OpCode::OpSub,
            OpCode::OpConstant,
            OpCode::Byte(3),
            OpCode::OpMult,
            OpCode::OpAdd,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
        compiler.compare_chunk(&expected_code, &expected_value_array);
    }

    #[test]
    fn parse_nested_variant_two() {
        let compiler = test_compiler_init_and_compile(String::from("(1 + 3 * 4) + 5"));
        let expected_code: Vec<OpCode> = Vec::from([
            OpCode::OpConstant,
            OpCode::Byte(0),
            OpCode::OpConstant,
            OpCode::Byte(1),
            OpCode::OpConstant,
            OpCode::Byte(2),
            OpCode::OpMult,
            OpCode::OpAdd,
            OpCode::OpConstant,
            OpCode::Byte(3),
            OpCode::OpAdd,
            OpCode::OpReturn,
        ]);
        let expected_value_array: [f64; 4] = [1.0, 3.0, 4.0, 5.0];
        compiler.compare_chunk(&expected_code, &expected_value_array);
    }
}
