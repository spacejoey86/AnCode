use clap::Parser;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::io::Error;

// #[command(author, version)]
#[derive(Parser, Debug)]
struct Args {
    entry_file: String,
}

struct Token {
    token_type: TokenType,
    value: String,
    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,
}

struct LexError {
    error_type: LexErrorType,
    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,
    file: String,
}
enum LexErrorType {
    WrongQuotes,
    MalformedBinLiteral,
    WrongHexCase,
    MalformedHexLiteral,
    MalformedDecLiteral,
    MultipleDecimalPoints,
    UnexpectedCharacter,
}

enum TokenType {
    BinLiteral,
    HexLiteral,
    DecimalLiteral(bool), //has_decimal_point
    StringLiteral(bool), //next_char_escaped
    Operator(Operator),
    LineComment,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Identifier,
}

impl TokenType {
}

enum Operator {
    Plus,
    Minus,
    Times,
    Divide,
}

struct Lexer {
    full_tokens: Vec<Token>,
    partial_token: String,
    // current_char: Option<char>,
    proposed_token_type: Option<TokenType>,

    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,

    file: String,
}

impl Lexer {
    fn new(current_file: String) -> Lexer{
        return Lexer {
            full_tokens: Vec::new(),
            partial_token: String::new(),
            // current_char: None,
            proposed_token_type: None,

            start_line: 0,
            end_line: 0,
            start_index: 0,
            end_index: 0,

            file: current_file,
        }
    }

    fn lex(mut self, source: String) -> Result<Vec<Token>,LexError> {
        for current_char in source.chars() {
            match self.consume_char(current_char) {
                Ok(()) => {},
                Err(lex_error) => {
                    return Err(lex_error);
                }
            }
        }
        return Ok(self.full_tokens)
    }

    fn push_token(&mut self) {
        self.full_tokens.push(Token {
            token_type: std::mem::take(&mut self.proposed_token_type).expect("push called before token was type was decided"),
            value: std::mem::take(&mut self.partial_token),
            start_line: self.start_line, end_line: self.end_line, start_index: self.start_index, end_index: self.end_index });
        self.start_line = self.end_line;
        self.start_index = self.end_index;
        self.proposed_token_type = None;
    }

    fn push_char(&mut self, c: char) {
        self.partial_token.push(c);
        if c == '\n' {
            self.end_line += 1;
            self.end_index = 0;
        } else {
            self.end_index += 1;
        }
    }

    fn construct_error(&self, e_type: LexErrorType) -> LexError {
        return LexError { error_type: e_type,
            start_line: self.start_line, end_line: self.end_line,
            start_index: self.start_index, end_index: self.end_index,
            file: self.file.clone() }
    }

    fn consume_char(&mut self, current_char: char) -> Result<(), LexError>{
        match &self.proposed_token_type {
            Some(TokenType::BinLiteral) => {
                if "01".contains(current_char) {
                    self.push_char(current_char);
                    Ok(())
                } else if " \n".contains(current_char) { //TODO: What if the literal is followed by an operator
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    return Err(self.construct_error(LexErrorType::MalformedBinLiteral))
                }
            },
            Some(TokenType::HexLiteral) => {
                if "0123456789abcdef".contains(current_char) {
                    self.push_char(current_char);
                    Ok(())
                } else if "ABCDEF".contains(current_char) {
                    return Err(self.construct_error(LexErrorType::WrongHexCase))
                } else if " \n".contains(current_char) {
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    return Err(self.construct_error(LexErrorType::MalformedHexLiteral))
                }
            },
            Some(TokenType::DecimalLiteral(has_decimal_point)) => {
                if self.partial_token == "0" {
                    if current_char == 'b' {
                        self.proposed_token_type = Some(TokenType::BinLiteral);
                        self.push_char(current_char);
                        return Ok(())
                    } else if current_char == 'x' {
                        self.proposed_token_type = Some(TokenType::HexLiteral);
                        self.push_char(current_char);
                        return Ok(())
                    }

                }
                if "0123456789".contains(current_char) {
                    self.push_char(current_char);
                    Ok(())
                } else if current_char == '.' {
                    if *has_decimal_point {
                            return Err(self.construct_error(LexErrorType::MultipleDecimalPoints))
                    } else {
                        self.proposed_token_type = Some(TokenType::DecimalLiteral(true));
                        self.push_char(current_char);
                        Ok(())
                    }
                } else if " \n".contains(current_char) {
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    return Err(self.construct_error(LexErrorType::MalformedDecLiteral))
                }
            },
            Some(TokenType::StringLiteral(escaped)) => {
                if (current_char == '"') && (! escaped) {
                    self.push_char(current_char);
                    self.push_token();
                    return Ok(())
                } else {
                    self.push_char(current_char);
                    return Ok(());
                }
            },
            Some(TokenType::LineComment) => {
                if current_char == '\n' {
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    self.push_char(current_char);
                    return Ok(())
                }
            },
            Some(TokenType::Operator(op)) => {
                match op {
                    Operator::Divide => {
                        if current_char == '/' {
                            self.proposed_token_type = Some(TokenType::LineComment);
                            self.push_char(current_char);
                            return Ok(())
                        } else {
                            self.push_token();
                            return self.consume_char(current_char);
                        }
                    },
                    _ => {
                        panic!("unterminated operator token")
                    }
                }
            }
            Some(TokenType::Identifier) => {
                if " \n".contains(current_char) {
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    self.push_char(current_char);
                    return Ok(());
                }
            },
            Some(TokenType::LeftBrace) | Some(TokenType::RightBrace) |
            Some(TokenType::LeftParen) | Some(TokenType::RightParen) => {
                panic!("Unexpected partial bracket token")
            }
            None => {
                match current_char {
                    '0'..='9' => {
                        self.push_char(current_char);
                        self.push_token();
                        self.proposed_token_type = Some(TokenType::DecimalLiteral(false));
                        return Ok(())
                    },
                    '"' => {
                        self.proposed_token_type = Some(TokenType::StringLiteral(false));
                        return Ok(())
                    },
                    '\'' => {
                        return Err(self.construct_error(LexErrorType::WrongQuotes))
                    },
                    '+' => {
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Plus));
                        return Ok(())
                    },
                    '-' => {
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Minus));
                        return Ok(())
                    },
                    '*' => {
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Times));
                        return Ok(())
                    },
                    '/' => {
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Divide));
                        return Ok(())
                    },

                    '(' => {
                        self.proposed_token_type = Some(TokenType::LeftParen);
                        return Ok(())
                    },
                    ')' => {
                        self.proposed_token_type = Some(TokenType::RightParen);
                        return Ok(())
                    },
                    '{' => {
                        self.proposed_token_type = Some(TokenType::LeftBrace);
                        return Ok(())
                    },
                    '}' => {
                        self.proposed_token_type = Some(TokenType::RightBrace);
                        return Ok(())
                    },

                    'a'..='z' | 'A'..='Z' => {
                        self.proposed_token_type = Some(TokenType::Identifier);
                        return Ok(())
                    },
                    _ => {
                        return Err(self.construct_error(LexErrorType::UnexpectedCharacter))
                    }
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("Compiling {}", args.entry_file);

    let entry_file_result = File::open(&args.entry_file);
    match entry_file_result {
        Ok(mut main_file) => {
            println!("Found file {}", args.entry_file);
            //do compiler stuff here
            let mut file_string = String::new();
            let file_result = main_file.read_to_string(&mut file_string);
            match file_result {
                Ok(_) => {
                    println!("Lexing...");
                    let lexer = Lexer::new(args.entry_file);
                    let tokens_result: Result<Vec<Token>,LexError> = lexer.lex(file_string);
                },
                Err(file_error) => {
                    deal_with_file_error(file_error, args.entry_file)
                }
            }
        },
        Err(error) => {
            deal_with_file_error(error, args.entry_file);
        }
    }
}

fn deal_with_file_error(file_error: Error, file_name: String) {
    match file_error.kind() {
        ErrorKind::NotFound => {
            println!("Could not find main file '{}'", file_name);
        },
        ErrorKind::PermissionDenied => {
            println!("Permission denied to open main file '{}'", file_name);
        },
        ErrorKind::Other |
        _ => {
            println!("Unknown error opening main file '{}'", file_name);
            println!("{}", file_error);
        }
    }
}