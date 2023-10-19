pub struct Token {
    token_type: TokenType,
    value: String,
    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,
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
    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,
    file: String,
    file_contents: String
}
impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error while lexing file {}\n", self.file)?;

        let underline: String;
        let line: String;
        let line_num = if self.start_line == self.end_line {
            //single line error:
            line = self.file_contents.lines().nth(self.start_line as usize - 1).unwrap().to_string();
            underline = " ".repeat(self.start_index as usize) +
                &"^".repeat((self.end_index - self.start_index) as usize) +
                &"\n";
            self.start_line.to_string()
        } else {
            //multi-line error
            line = self.file_contents.lines()
                .skip(self.start_line as usize - 1)
                .take((self.end_line - self.start_line) as usize)
                .map(|x| x.to_owned().chars().collect::<Vec<char>>())
                .flatten().collect();
            underline = "".into();
            self.start_line.to_string() + "-" + &self.end_line.to_string()
        };
        let index_num = if self.start_index == self.end_index {
            self.start_index.to_string()
        } else {
            self.start_index.to_string() + "-" + &self.end_index.to_string()
        };

        write!(f, "{} on line {}, index {}:\n", self.error_type.to_string(), line_num, index_num)?;
        write!(f, "{}\n{}", line, underline)
    }
}

#[derive(Debug)]
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

    start_line: i32,
    end_line: i32,
    start_index: i32,
    end_index: i32,

    file: String,
    file_contents: Option<String>
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
                } else if " \n".contains(current_char) { //TODO: What if the literal is followed by an operator
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
                } else if " \n".contains(current_char) {
                    self.push_token();
                    return self.consume_char(current_char);
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
                } else if " \n".contains(current_char) {
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
                if " \n".contains(current_char) {
                    self.push_token();
                    return self.consume_char(current_char);
                } else {
                    self.push_char(current_char);
                    return Ok(());
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
                panic!("Unexpected partial bracket token")
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

    fn lex_to_tokens(source: &str) -> Vec<TokenType>{
        let lexer = Lexer::new("my_file".into());
        let tokens = lexer.lex(source.into()).expect("Unexpected error during test");
        return tokens.iter().map(|x| x.token_type).collect();
    }

    #[test]
    fn single_identifier() {
        assert_eq!(lex_to_tokens("MyVariable\n"),
            vec![TokenType::Identifier, TokenType::Newline, TokenType::EndOfFile]);
    }
}
