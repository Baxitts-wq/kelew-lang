use crate::token::{Span, Token, TokenType};

pub struct Lexer {
    chars: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
    start_col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            col: 1,
            start_col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            self.start_col = self.col;

            if let Some(token) = self.scan_token()? {
                tokens.push(token);
            }
        }

        tokens.push(Token::new(TokenType::Eof, Span::new(self.line, self.col)));
        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Option<Token>, String> {
        let c = self.advance();

        match c {
            ' ' | '\r' | '\t' => Ok(None),
            '\n' => {
                self.line += 1;
                self.col = 1;
                Ok(None)
            }
            // Commentaires # ou //
            '#' => {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
                Ok(None)
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(None)
                } else {
                    Ok(Some(self.make_token(TokenType::Slash)))
                }
            }
            '(' => Ok(Some(self.make_token(TokenType::LeftParen))),
            ')' => Ok(Some(self.make_token(TokenType::RightParen))),
            ',' => Ok(Some(self.make_token(TokenType::Comma))),
            '+' => Ok(Some(self.make_token(TokenType::Plus))),
            '-' => Ok(Some(self.make_token(TokenType::Minus))),
            '*' => Ok(Some(self.make_token(TokenType::Star))),
            '%' => Ok(Some(self.make_token(TokenType::Modulo))),
            '=' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::EqualEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Equal)))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::BangEqual)))
                } else {
                    Err(format!("Caractère inattendu '!' à la ligne {}, colonne {}", self.line, self.start_col))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::LessEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Less)))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::GreaterEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Greater)))
                }
            }
            '"' => self.string(),
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier_or_keyword(),
            _ => Err(format!(
                "Caractère non reconnu '{}' à la ligne {}, colonne {}",
                c, self.line, self.start_col
            )),
        }
    }

    fn string(&mut self) -> Result<Option<Token>, String> {
        let mut value = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 1;
            }
            if self.peek() == '\\' {
                self.advance();
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    other => value.push(other),
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(format!(
                "Chaîne de caractères non fermée à la ligne {}, colonne {}",
                self.line, self.start_col
            ));
        }

        // Consommer le guillemet fermant '"'
        self.advance();
        Ok(Some(self.make_token(TokenType::String(value))))
    }

    fn number(&mut self) -> Result<Option<Token>, String> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // Consomme '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let literal: String = self.chars[self.start..self.current].iter().collect();
        let value: f64 = literal
            .parse()
            .map_err(|_| format!("Nombre invalide '{}' à la ligne {}", literal, self.line))?;

        Ok(Some(self.make_token(TokenType::Number(value))))
    }

    fn identifier_or_keyword(&mut self) -> Result<Option<Token>, String> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.chars[self.start..self.current].iter().collect();
        let lower = text.to_lowercase();

        // 1. Détection "T3boun dymek" / "Tboun dymek"
        if lower == "t3boun" || lower == "tboun" {
            let saved_curr = self.current;
            let saved_col = self.col;
            let saved_line = self.line;

            self.skip_whitespace();
            let next_word = self.peek_alphanumeric_word();
            if next_word.as_deref().map(|s| s.to_lowercase()) == Some("dymek".to_string()) {
                self.consume_word(&next_word.unwrap());
                return Ok(Some(self.make_token(TokenType::T3bounDymek)));
            } else {
                self.current = saved_curr;
                self.col = saved_col;
                self.line = saved_line;
            }
        }

        // 2. Détection "Fio del puta"
        if lower == "fio" {
            let saved_curr = self.current;
            let saved_col = self.col;
            let saved_line = self.line;

            self.skip_whitespace();
            let w2 = self.peek_alphanumeric_word();
            if w2.as_deref().map(|s| s.to_lowercase()) == Some("del".to_string()) {
                let w2_str = w2.unwrap();
                self.consume_word(&w2_str);
                self.skip_whitespace();
                let w3 = self.peek_alphanumeric_word();
                if w3.as_deref().map(|s| s.to_lowercase()) == Some("puta".to_string()) {
                    self.consume_word(&w3.unwrap());
                    return Ok(Some(self.make_token(TokenType::FioDelPuta)));
                }
            }
            self.current = saved_curr;
            self.col = saved_col;
            self.line = saved_line;
        }

        // Mots-clés simples
        let token_type = match lower.as_str() {
            "kelew" => TokenType::Kelew,
            "ejacule" => TokenType::Ejacule,
            "jib" | "chouf" => TokenType::Jib,
            "zaama" => TokenType::Zaama,
            "walou" => TokenType::Walou,
            "khalas" => TokenType::Khalas,
            "dima" => TokenType::Dima,
            "baraka" => TokenType::Baraka,
            "sah" => TokenType::Sah,
            "k7hba" | "k7ba" | "kahba" | "khedba" => TokenType::K7hba,
            _ => TokenType::Identifier(text),
        };

        Ok(Some(self.make_token(token_type)))
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && (self.peek() == ' ' || self.peek() == '\t' || self.peek() == '\r') {
            self.advance();
        }
    }

    fn peek_alphanumeric_word(&self) -> Option<String> {
        let mut idx = self.current;
        while idx < self.chars.len() && (self.chars[idx] == ' ' || self.chars[idx] == '\t' || self.chars[idx] == '\r') {
            idx += 1;
        }
        if idx >= self.chars.len() {
            return None;
        }

        let start = idx;
        while idx < self.chars.len() && (self.chars[idx].is_alphanumeric() || self.chars[idx] == '_') {
            idx += 1;
        }

        if start == idx {
            None
        } else {
            Some(self.chars[start..idx].iter().collect())
        }
    }

    fn consume_word(&mut self, word: &str) {
        self.skip_whitespace();
        for _ in 0..word.chars().count() {
            self.advance();
        }
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        self.col += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            false
        } else {
            self.current += 1;
            self.col += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, Span::new(self.line, self.start_col))
    }
}
