use clap::Parser;
use utf8_chars::BufReadCharsExt;
use std::fs::File;
use std::io::{ErrorKind, BufReader};

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
    error_description: String,
    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,
}

enum TokenType {
    DecimalLiteral,
    HexLiteral,
    BinLiteral,

}

fn main() {
    let args = Args::parse();
    println!("Compiling {}", args.entry_file);

    let entry_file_result = File::open(&args.entry_file);
    match entry_file_result {
        Ok(main_file) => {
            println!("Found file {}", args.entry_file);
            //do compiler stuff here
            let tokens: Vec<Token> = lex(main_file);
        },
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("Could not find main file '{}'", args.entry_file);
            },
            ErrorKind::PermissionDenied => {
                println!("Permission denied to open main file '{}'", args.entry_file);
            },
            ErrorKind::Other |
            _ => {
                println!("Unknown error opening main file '{}'", args.entry_file);
                println!("{}", error);
            }
        }
    }
}



fn lex(source_file: File) -> Result<Vec<Token>, LexError> {
    let mut file_reader = BufReader::new(source_file);

    let mut tokens = Vec::<Token>::new();
    let mut token = String::new();
    let mut current = "";
    let mut valid_token: Option<TokenType> =  None;

    let mut start_line = 0;
    let mut end_line = 0;
    let mut start_index = 0;
    let mut end_index = 0;

    for c in file_reader.chars().map(|x| x.unwrap()) {
        current = &c.to_string();
        while current != "" {
            match valid_token {
                Some(TokenType::BinLiteral) => {
                    if "01".contains(current) {
                        token = token.to_owned() + &current.to_string();
                        current = "";
                        end_index += 1;
                        continue;
                    } else {
                        tokens.push(Token { token_type: TokenType::BinLiteral, value: token, start_line: start_line, end_line: end_line, start_index: start_index, end_index: end_index });
                        valid_token = None;
                        token = String::new();
                        start_line = end_line;
                        start_index = end_index;
                    }
                },
                Some(TokenType::HexLiteral) => {
                    if "0123456789abcdef".contains(current) {
                        token = token.to_owned() + &current.to_string();
                        current = "";
                        end_index += 1;
                        continue;
                    } else if "ABCDEF".contains(current) {
                        return Err(LexError { error_description: "Hexadecimal literal should be in uppercase".to_owned(),
                        start_line: start_line, end_line: end_line, start_index: start_index, end_index: end_index })
                    }
                }
            }
        }


        if "01".contains(c) {
            match valid_token {
                Some(TokenType::BinLiteral) => {
                    token = token.to_owned() + &c.to_string();
                    continue;
                }
            }
        }
        if "0123456789".contains(c) {
            match valid_token {
                
            }
        }


        match c {
            c if "0123456789".contains(c) => {
                match valid_token {
                    None => {
                        valid_token = Some(TokenType::DecimalLiteral)
                    },
                    Some(t) => {
                        match t {
                            TokenType::BinLiteral => {
                                match c {
                                    '0' | '1' => {
                                        token = &(token.to_owned() + &c.to_string())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return tokens;
}