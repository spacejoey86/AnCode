#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    value: String,
    start_line: usize,
    end_line: usize,
    start_index: usize,
    end_index: usize,
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: \"{}\"", self.token_type, self.value)
    }
}

#[derive(Debug)]
pub struct LexError {
    error_type: LexErrorType,
    partial_token: String,
    start_line: usize,
    end_line: usize,
    start_index: usize,
    end_index: usize,
    file: String,
    file_contents: String
}
impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error while lexing file {}\n", self.file)?;

        let index_num = if self.start_index == self.end_index {
            self.start_index.to_string()
        } else {
            self.start_index.to_string() + "-" + &self.end_index.to_string()
        };

        let underline: String;
        let line: String;
        let line_num = if self.start_line == self.end_line {
            //single line error:
            line = self.file_contents.lines().nth(self.start_line - 1).unwrap().to_string();
            underline = " ".repeat(self.start_index as usize) +
                &"^".repeat(self.end_index - self.start_index) +
                &"\n";
            "line ".to_string() + &self.start_line.to_string() + ", index " + &index_num
        } else {
            //multi-line error
            line = self.file_contents.lines()
                .skip(self.start_line - 1)
                .take(self.end_line - self.start_line + 1)
                .map(|x| x.to_owned()).collect::<Vec<String>>()
                .join("\n");
            underline = "".into();
            "lines ".to_string() + &self.start_line.to_string() + "-" + &self.end_line.to_string()
        };

        write!(f, "{} on {}:\n", self.error_type.to_string(), line_num)?;
        write!(f, "{}\n{}", line, underline)
    }
}

#[derive(Debug, PartialEq)]
enum LexErrorType {
    WrongQuotes,
    MalformedBinLiteral,
    WrongHexCase,
    MalformedHexLiteral,
    MalformedDecLiteral,
    MultipleDecimalPoints,
    UnexpectedCharacter,
    TrailingDPoint,
    EmptyBinLiteral,
    EmptyHexLiteral,
    UnexpectedEOFString,
    MissingTrailingNewLine,
}
impl std::fmt::Display for LexErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LexErrorType::WrongQuotes => write!(f, "Wrong quotes"),
            LexErrorType::MalformedBinLiteral => write!(f, "Malformed binary literal"),
            LexErrorType::WrongHexCase => write!(f, "Hexadecimals should always use lower case"),
            LexErrorType::MalformedHexLiteral => write!(f, "Malformed hexadecimal literal"),
            LexErrorType::MalformedDecLiteral => write!(f, "Malformed decimal literal"),
            LexErrorType::MultipleDecimalPoints => write!(f, "Multiple decimal points in decimal literal"),
            LexErrorType::UnexpectedCharacter => write!(f, "Unexpected character in input"),
            LexErrorType::TrailingDPoint => write!(f, "Decimal literal cannot end in decimal point"),
            LexErrorType::EmptyBinLiteral => write!(f, "Binary literal must be at least one bit long"),
            LexErrorType::EmptyHexLiteral => write!(f, "Hexadecimal literal must be at least one digit long"),
            LexErrorType::UnexpectedEOFString => write!(f, "Unexpected EOF while lexing string literal"),
            LexErrorType::MissingTrailingNewLine => write!(f, "File should end with a trailing newline"),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
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

    Equals,

    Identifier,

    Whitespace,
    Newline,
    EndOfFile,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::BinLiteral => write!(f, "Binary literal"),
            TokenType::HexLiteral => write!(f, "Hexadecimal literal"),
            TokenType::DecimalLiteral(_) => write!(f, "Decimal literal"),
            TokenType::StringLiteral(_) => write!(f, "String literal"),
            TokenType::Operator(Operator::Plus) => write!(f, "Plus operator"),
            TokenType::Operator(Operator::Minus) => write!(f, "Minus operator"),
            TokenType::Operator(Operator::Multiply) => write!(f, "Multiply operator"),
            TokenType::Operator(Operator::Divide) => write!(f, "Divide operator"),
            TokenType::Operator(Operator::Equals) => write!(f, "Equality operator"),
            TokenType::LineComment => write!(f, "Line comment"),
            TokenType::LeftParen => write!(f, "Left paren"),
            TokenType::RightParen => write!(f, "Right paren"),
            TokenType::LeftBrace => write!(f, "Left brace"),
            TokenType::RightBrace => write!(f, "Right brace"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::Newline => write!(f, "Newline"),
            TokenType::EndOfFile => write!(f, "End of file"),
            TokenType::Equals => write!(f, "Equals"),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
}

pub struct Lexer {
    full_tokens: Vec<Token>,
    partial_token: String,
    current_char: Option<char>,
    proposed_token_type: Option<TokenType>,

    start_line: usize,
    end_line: usize,
    start_index: usize,
    end_index: usize,

    file: String,
    file_contents: Option<String>
}

fn is_literal_terminator(current_char: char) -> bool {
    if "+-*/!\"%^&(){}[].,|:; \n".contains(current_char) {
        return true;
    } else {
        return false;
    }
}

impl Lexer {
    pub fn new(current_file: String) -> Lexer{
        return Lexer {
            full_tokens: Vec::new(),
            partial_token: String::new(),
            current_char: None,
            proposed_token_type: None,

            start_line: 1,
            end_line: 1,
            start_index: 0,
            end_index: 0,

            file: current_file,
            file_contents: None,
        }
    }

    pub fn lex(mut self, source: String) -> Result<Vec<Token>,LexError> {
        self.file_contents = Some(source.clone());
        for current_char in source.chars() {
            match self.consume_char(current_char) {
                Ok(()) => {},
                Err(lex_error) => {
                    return Err(lex_error);
                }
            }
        }

        //partial token followed by EOF
        match self.proposed_token_type {
            Some(TokenType::StringLiteral(_)) => {
                return Err(self.construct_error(LexErrorType::UnexpectedEOFString))
            },
            None => {},
            Some(_) => {
                return Err(self.construct_error(LexErrorType::MissingTrailingNewLine))
            }
        }
        self.proposed_token_type = Some(TokenType::EndOfFile);
        self.push_token();

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
        let token = self.partial_token.clone();
        return LexError { error_type: e_type, partial_token: token,
            start_line: self.start_line, end_line: self.end_line,
            start_index: self.start_index, end_index: self.end_index,
            file: self.file.clone(), file_contents: self.file_contents.clone().unwrap()}
    }

    fn construct_error_w_char(&mut self, e_type: LexErrorType) -> LexError {
        self.partial_token.push(self.current_char.unwrap_or_default());
        return self.construct_error(e_type);
    }

    fn consume_char(&mut self, current_char: char) -> Result<(), LexError>{
        self.current_char = Some(current_char);
        match &self.proposed_token_type {
            Some(TokenType::BinLiteral) => {
                if "01".contains(current_char) {
                    self.push_char(current_char);
                    Ok(())
                } else if is_literal_terminator(current_char) { //TODO: What if the literal is followed by an operator
                    match self.partial_token.chars().last().unwrap() {
                        'b' => {
                            return Err(self.construct_error_w_char(LexErrorType::EmptyBinLiteral))
                        },
                        _ => {
                            self.push_token();
                            return self.consume_char(current_char);
                        }
                    }
                } else {
                    return Err(self.construct_error_w_char(LexErrorType::MalformedBinLiteral))
                }
            },
            Some(TokenType::HexLiteral) => {
                if "0123456789abcdef".contains(current_char) {
                    self.push_char(current_char);
                    Ok(())
                } else if "ABCDEF".contains(current_char) {
                    return Err(self.construct_error_w_char(LexErrorType::WrongHexCase))
                } else if is_literal_terminator(current_char) {
                    if self.partial_token.chars().last().unwrap() == 'x' {
                        return Err(self.construct_error(LexErrorType::EmptyHexLiteral));
                    } else {
                        self.push_token();
                        return self.consume_char(current_char);
                    }
                } else {
                    return Err(self.construct_error_w_char(LexErrorType::MalformedHexLiteral))
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
                            return Err(self.construct_error_w_char(LexErrorType::MultipleDecimalPoints))
                    } else {
                        self.proposed_token_type = Some(TokenType::DecimalLiteral(true));
                        self.push_char(current_char);
                        Ok(())
                    }
                } else if is_literal_terminator(current_char) {
                    match self.partial_token.chars().last().unwrap() {
                        '.' => {
                            return Err(self.construct_error_w_char(LexErrorType::TrailingDPoint))
                        },
                        _ => {
                            self.push_token();
                            return self.consume_char(current_char);
                        }
                    }
                } else {
                    return Err(self.construct_error_w_char(LexErrorType::MalformedDecLiteral))
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
            },
            Some(TokenType::Whitespace) => {
                if current_char == ' ' {
                    self.push_char(current_char);
                    Ok(())
                } else {
                    self.push_token();
                    return self.consume_char(current_char);
                }
            },
            Some(TokenType::Identifier) => {
                if current_char.is_alphabetic() {
                    self.push_char(current_char);
                    return Ok(());
                } else {
                    self.push_token();
                    return self.consume_char(current_char);

                }
            },
            Some(TokenType::Equals) => {
                if current_char == '=' {
                    self.proposed_token_type = Some(TokenType::Operator(Operator::Equals));
                    self.push_char(current_char);
                    self.push_token();
                    Ok(())
                } else {
                    self.push_token();
                    return self.consume_char(current_char);
                }
            }
            Some(TokenType::LeftBrace) | Some(TokenType::RightBrace) |
            Some(TokenType::LeftParen) | Some(TokenType::RightParen) |
            Some(TokenType::Newline) | Some(TokenType::EndOfFile) => {
                panic!("Unexpected partial token")
            }
            None => {
                match current_char {
                    '0'..='9' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::DecimalLiteral(false));
                        return Ok(())
                    },
                    '"' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::StringLiteral(false));
                        return Ok(())
                    },
                    '\'' => {
                        return Err(self.construct_error_w_char(LexErrorType::WrongQuotes))
                    },
                    '+' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Plus));
                        self.push_token();
                        return Ok(())
                    },
                    '-' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Minus));
                        self.push_token();
                        return Ok(())
                    },
                    '*' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Multiply));
                        self.push_token();
                        return Ok(())
                    },
                    '/' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Operator(Operator::Divide));
                        return Ok(())
                    },

                    '(' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::LeftParen);
                        self.push_token();
                        return Ok(())
                    },
                    ')' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::RightParen);
                        self.push_token();
                        return Ok(())
                    },
                    '{' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::LeftBrace);
                        self.push_token();
                        return Ok(())
                    },
                    '}' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::RightBrace);
                        self.push_token();
                        return Ok(())
                    },

                    'a'..='z' | 'A'..='Z' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Identifier);
                        return Ok(())
                    },
                    ' ' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Whitespace);
                        return Ok(())
                    },
                    '\n' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Newline);
                        self.push_token();
                        return Ok(())
                    },
                    '=' => {
                        self.push_char(current_char);
                        self.proposed_token_type = Some(TokenType::Equals);
                        return Ok(());
                    }
                    _ => {
                        return Err(self.construct_error_w_char(LexErrorType::UnexpectedCharacter))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_to_tokens(source: &str) -> Vec<TokenType> {
        let lexer = Lexer::new("my_file".into());
        let tokens = lexer.lex(source.into()).expect("Unexpected error during test");
        return tokens.iter().map(|x| x.token_type).collect();
    }

    fn lex_to_err(source: &str) -> LexErrorType {
        let lexer = Lexer::new("my_file".into());
        match lexer.lex(source.into()) {
            Ok(_) => {
                panic!("Error not thrown when expected");
            },
            Err(e) => {
                return e.error_type;
            }
        }
    }

    fn lex(source: &str) -> Result<Vec<Token>, LexError>{
        let lexer = Lexer::new("my_file".into());
        return lexer.lex(source.into())
    }

    #[test]
    fn single_identifier() {
        assert_eq!(lex_to_tokens("MyVariable\n"),
            vec![TokenType::Identifier, TokenType::Newline, TokenType::EndOfFile]);
    }


    // Test the various errors
    #[test]
    fn wrong_quotes() {
        assert_eq!(lex_to_err("'Hello world'"), LexErrorType::WrongQuotes)
    }

    #[test]
    fn wrong_quote() {
        assert_eq!(lex_to_err("'Hello wo"), LexErrorType::WrongQuotes)
    }

    #[test]
    fn wrong_quote_inside() {
        match lex("\"don't want an error here\"") {
            Ok(_) => {},
            Err(e) => {
                panic!("Incorrectly errors on single quote within string literal")
            }
        }
    }

    #[test]
    fn malformed_binary() {
        assert_eq!(lex_to_err("0b0110534"), LexErrorType::MalformedBinLiteral)
    }

    #[test]
    fn hex_right() {
        match lex("0xdeadbeef\n") {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e);
                panic!("Incorrectly errors on correct hex literal")
            }
        }
    }

    #[test]
    fn hex_wrong() {
        assert_eq!(lex_to_err("0x4D\n"), LexErrorType::WrongHexCase);
    }

    #[test]
    fn hex_mixed() {
        assert_eq!(lex_to_err("0x4Dd\n"), LexErrorType::WrongHexCase);
    }

    #[test]
    fn bad_hex() {
        assert_eq!(lex_to_err("0x4dk\n"), LexErrorType::MalformedHexLiteral);
    }

    #[test]
    fn dec_wrong() {
        assert_eq!(lex_to_err("0.f\n"), LexErrorType::MalformedDecLiteral);
    }

    #[test]
    fn dec_trailing_dpoint() {
        assert_eq!(lex_to_err("56.\n"), LexErrorType::TrailingDPoint);
    }

    #[test]
    fn dec_multiple_dpoint() {
        assert_eq!(lex_to_err("7.3.7"), LexErrorType::MultipleDecimalPoints);
    }

    #[test]
    fn malformed_decimal() {
        assert_eq!(lex_to_err("56j54"), LexErrorType::MalformedDecLiteral);
    }

    #[test]
    fn decimal_and_operators() {
        match lex("56+23\n") {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e);
                panic!("Decimal not terminated at operator")
            }
        }
    }

    #[test]
    fn bin_empty() {
        assert_eq!(lex_to_err("0b\n"), LexErrorType::EmptyBinLiteral);
    }

    #[test]
    fn hex_empty() {
        assert_eq!(lex_to_err("0x\n"), LexErrorType::EmptyHexLiteral);
    }

    #[test]
    fn unexpected_end_of_file() {
        assert_eq!(lex_to_err("\"Hello wo"), LexErrorType::UnexpectedEOFString);
    }

    #[test]
    fn trailing_newline() {
        assert_eq!(lex_to_err("let x = 4"), LexErrorType::MissingTrailingNewLine);
    }
}
