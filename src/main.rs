use clap::Parser;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::io::Error;

mod lexer;
use crate::lexer::{Token, LexError, Lexer};

// #[command(author, version)]
#[derive(Parser, Debug)]
struct Args {
    entry_file: String,
    #[arg(short, long)]
    lexer_debug: bool,
}

fn main() {
    let args = Args::parse();

    let entry_file_result = File::open(&args.entry_file);
    match entry_file_result {
        Ok(mut main_file) => {
            //do compiler stuff here
            let mut file_string = String::new();
            let file_result = main_file.read_to_string(&mut file_string);
            match file_result {
                Ok(_) => {
                    let lexer = Lexer::new(args.entry_file);
                    let tokens_result: Result<Vec<Token>,LexError> = lexer.lex(file_string);

                    match tokens_result {
                        Ok(tokens) => {
                            if args.lexer_debug {
                                println!("There are {} tokens", tokens.len());
                                println!("[DEBUG] Tokens:");
                                for token in tokens {
                                    println!("{}", token)
                                }
                            }
                        },
                        Err(lex_error) => {
                            print!("{}", lex_error.to_string())
                        }
                    }

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