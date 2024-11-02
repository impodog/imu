use super::*;
use crate::token::*;

type TokenAC = AhoCorasick<TokenKind>;
lazy_static::lazy_static! {
    static ref KEYWORDS: TokenAC = {
        let mut ac = AhoCorasickBuilder::default();
        ac.insert("pub", TokenKind::Keyword(Keyword::Pub));
        ac.insert("mut", TokenKind::Keyword(Keyword::Mut));
        ac.insert("fun", TokenKind::Keyword(Keyword::Fun));
        ac.insert("cus", TokenKind::Keyword(Keyword::Cus));
        ac.insert("add", TokenKind::Keyword(Keyword::Add));
        ac.insert("for", TokenKind::Keyword(Keyword::For));
        ac.insert("new", TokenKind::Keyword(Keyword::New));
        ac.insert("rep", TokenKind::Keyword(Keyword::Rep));
        ac.insert("if", TokenKind::Keyword(Keyword::If));
        ac.insert("else", TokenKind::Keyword(Keyword::Else));
        ac.insert("true", TokenKind::ResVal(ResVal::True));
        ac.insert("false", TokenKind::ResVal(ResVal::False));
        ac.insert("self", TokenKind::ResVal(ResVal::SelfValue));
        ac.insert("Self", TokenKind::ResTy(ResTy::SelfType));
        ac.insert("I8", TokenKind::ResTy(ResTy::I8));
        ac.insert("I16", TokenKind::ResTy(ResTy::I16));
        ac.insert("I32", TokenKind::ResTy(ResTy::I32));
        ac.insert("I64", TokenKind::ResTy(ResTy::I64));
        ac.insert("Ptr", TokenKind::ResTy(ResTy::Ptr));
        ac.insert("F32", TokenKind::ResTy(ResTy::F32));
        ac.insert("F64", TokenKind::ResTy(ResTy::F64));
        ac.insert("Str", TokenKind::ResTy(ResTy::Str));
        ac.insert("I128", TokenKind::ResTy(ResTy::I128));
        ac.build()
    };
}

impl<I> Iterator for Reader<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

impl<I> Reader<I>
where
    I: Iterator<Item = char>,
{
    /// Scans the next token of this reader
    pub(crate) fn next_token(&mut self) -> Token {
        let begin = self.cursor();
        let ch = self.next_char();

        if ch.is_whitespace() {
            if ch == '\r' {
                Token::new(TokenKind::Stray, self.diff(begin))
            } else if ch == '\n' {
                Token::new(TokenKind::Spacing(Spacing::LineBreak), self.diff(begin))
            } else {
                Token::new(self.next_indent(), self.diff(begin))
            }
        } else if ch.is_ascii_digit() {
            Token::new(self.next_number(ch), self.diff(begin))
        } else if ch.is_lowercase() {
            Token::new(self.next_value(ch), self.diff(begin))
        } else if ch.is_uppercase() {
            Token::new(self.next_type(ch), self.diff(begin))
        } else {
            match ch {
                EOF => Token::new(TokenKind::Eof, self.diff(begin)),

                '_' => Token::new(self.next_unused(), self.diff(begin)),
                '\"' => Token::new(self.next_string(), self.diff(begin)),

                '(' => Token::new(TokenKind::Bracket(Pair::LeftParen), self.diff(begin)),
                ')' => Token::new(TokenKind::Bracket(Pair::RightParen), self.diff(begin)),
                '[' => Token::new(TokenKind::Bracket(Pair::LeftBracket), self.diff(begin)),
                ']' => Token::new(TokenKind::Bracket(Pair::RightBracket), self.diff(begin)),
                '{' => Token::new(TokenKind::Bracket(Pair::LeftBrace), self.diff(begin)),
                '}' => Token::new(TokenKind::Bracket(Pair::RightBrace), self.diff(begin)),

                '.' => Token::new(TokenKind::BinOp(BinOp::Dot), self.diff(begin)),
                ':' => Token::new(TokenKind::BinOp(BinOp::Colon), self.diff(begin)),
                '+' => Token::new(TokenKind::BinOp(BinOp::Add), self.diff(begin)),
                '-' => {
                    if self.first().is_ascii_digit() {
                        let ch = self.next_char();
                        Token::new(self.next_number(ch), self.diff(begin))
                    } else {
                        Token::new(TokenKind::UnOp(UnOp::Neg), self.diff(begin))
                    }
                }
                '*' => Token::new(TokenKind::BinOp(BinOp::Mul), self.diff(begin)),
                '/' => match self.first() {
                    '/' => Token::new(self.next_comment(), self.diff(begin)),
                    '*' => Token::new(self.next_multi_comment(), self.diff(begin)),
                    _ => Token::new(TokenKind::BinOp(BinOp::Div), self.diff(begin)),
                },
                '=' => Token::new(TokenKind::BinOp(BinOp::Assign), self.diff(begin)),
                ';' => Token::new(TokenKind::Semicolon, self.diff(begin)),
                _ => Token::new(TokenKind::LexError(LexError::UnknownChar), self.diff(begin)),
            }
        }
    }

    fn next_comment(&mut self) -> TokenKind {
        self.advance();
        self.advance_while(|reader| {
            let ch = reader.first();
            ch != '\n' && ch != EOF
        });
        TokenKind::Comment(Comment::Comment)
    }

    fn next_multi_comment(&mut self) -> TokenKind {
        self.advance();
        let mut depth = 1;
        self.advance_while(|reader| {
            // NOTE: Maybe this way is a little inefficient as every char gets scanned twice
            if reader.first() == '/' && reader.second() == '*' {
                reader.advance();
                depth += 1;
            } else if reader.first() == '*' && reader.second() == '/' {
                reader.advance();
                depth -= 1;
            }
            if reader.first() == EOF {
                // This causes depth != 0 early quit
                false
            } else {
                depth != 0
            }
        });
        // Skip the closing comment '/'
        self.advance();
        if depth != 0 {
            TokenKind::LexError(LexError::UnclosedComment)
        } else {
            TokenKind::Comment(Comment::MultiComment)
        }
    }

    fn next_indent(&mut self) -> TokenKind {
        self.advance_while(|reader| reader.first().is_whitespace());
        TokenKind::Spacing(Spacing::Indent)
    }

    fn next_number(&mut self, ch: char) -> TokenKind {
        match ch {
            '0' => match self.first() {
                '0'..='9' => TokenKind::LexError(LexError::NumberError),
                'b' => {
                    self.advance();
                    self.advance_while(|reader| ('0'..='1').contains(&reader.first()));
                    TokenKind::Literal(Literal::Integer)
                }
                'x' => {
                    self.advance();
                    self.advance_while(|reader| reader.first().is_ascii_hexdigit());
                    TokenKind::Literal(Literal::Integer)
                }
                '.' => {
                    self.advance();
                    self.advance_while(|reader| reader.first().is_ascii_digit());
                    TokenKind::Literal(Literal::Float)
                }
                _ => TokenKind::Literal(Literal::Integer),
            },
            '1'..='9' => {
                self.advance_while(|reader| reader.first().is_ascii_digit());
                if self.first() == '.' {
                    self.advance();
                    self.advance_while(|reader| reader.first().is_ascii_digit());
                    if self.first() == 'e' {
                        self.advance();
                        if self.first() == '+' || self.first() == '-' {
                            self.advance();
                        }
                        self.advance_while(|reader| reader.first().is_ascii_digit());
                    }
                    TokenKind::Literal(Literal::Float)
                } else {
                    TokenKind::Literal(Literal::Integer)
                }
            }
            _ => unreachable!(),
        }
    }

    fn next_string(&mut self) -> TokenKind {
        if self.first() == '\"' && self.second() == '\"' {
            let mut accum = 0;
            self.advance();
            self.advance();
            self.advance_while(|reader| {
                if accum == 3 {
                    return false;
                }
                let ch = reader.first();
                if ch == EOF {
                    // This causes accum != 3 early quit
                    false
                } else {
                    if ch == '\"' {
                        accum += 1;
                    } else {
                        accum = 0;
                    }
                    true
                }
            });
            if accum != 3 {
                TokenKind::LexError(LexError::UnclosedString)
            } else {
                TokenKind::Literal(Literal::MultiString)
            }
        } else {
            let mut escape = false;
            self.advance_while(|reader| {
                let ch = reader.first();
                if ch == EOF {
                    false
                } else if escape {
                    escape = false;
                    true
                } else if ch == '\\' {
                    escape = true;
                    true
                } else {
                    ch != '\"'
                }
            });
            if self.first() != '\"' {
                TokenKind::LexError(LexError::UnclosedString)
            } else {
                // Skip the closing quote
                self.advance();
                TokenKind::Literal(Literal::String)
            }
        }
    }

    fn next_value(&mut self, ch: char) -> TokenKind {
        let mut pos = KEYWORDS.query(0, ch);
        self.advance_while(|reader| {
            let ch = reader.first();
            if ch.is_alphanumeric() || ch == '_' {
                pos = KEYWORDS.query(pos, ch);
                true
            } else {
                false
            }
        });
        if let Some(token) = KEYWORDS.finish(pos) {
            *token
        } else {
            TokenKind::Ident(Ident::Value)
        }
    }

    fn next_type(&mut self, ch: char) -> TokenKind {
        let mut pos = KEYWORDS.query(0, ch);
        self.advance_while(|reader| {
            let ch = reader.first();
            if ch.is_alphanumeric() || ch == '_' {
                pos = KEYWORDS.query(pos, ch);
                true
            } else {
                false
            }
        });
        if let Some(token) = KEYWORDS.finish(pos) {
            *token
        } else {
            TokenKind::Ident(Ident::Type)
        }
    }

    fn next_unused(&mut self) -> TokenKind {
        self.advance_while(|reader| {
            let ch = reader.first();
            ch.is_alphanumeric() || ch == '_'
        });
        TokenKind::Ident(Ident::Unused)
    }
}
