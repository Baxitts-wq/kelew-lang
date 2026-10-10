use crate::ast::{BinaryOp, Expr, Program, Stmt, Value};
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        // Si le premier token est T3bounDymek, on le consomme
        if self.match_token(&[TokenType::T3bounDymek]) {
            // Début de programme détecté
        }

        while !self.is_at_end() && !self.check(&TokenType::FioDelPuta) {
            statements.push(self.statement()?);
        }

        // Consommer FioDelPuta si présent
        if self.match_token(&[TokenType::FioDelPuta]) {
            // Fin de programme
        }

        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenType::Kelew]) {
            self.var_declaration()
        } else if self.match_token(&[TokenType::Ejacule]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::Jib]) {
            self.input_statement()
        } else if self.match_token(&[TokenType::Zaama]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::Dima]) {
            self.while_statement()
        } else {
            self.expression_statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let span = self.previous().span;

        let name = match self.peek().token_type.clone() {
            TokenType::Identifier(id) => {
                self.advance();
                id
            }
            _ => {
                return Err(format!(
                    "Nom de variable attendu après 'Kelew' à la ligne {}, colonne {}",
                    self.peek().span.line,
                    self.peek().span.col
                ))
            }
        };

        let initializer = if self.match_token(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Value::Nil, span)
        };

        Ok(Stmt::VarDecl {
            name,
            initializer,
            span,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let span = self.previous().span;
        let expr = self.expression()?;
        Ok(Stmt::Print { expr, span })
    }

    fn input_statement(&mut self) -> Result<Stmt, String> {
        let span = self.previous().span;
        let name = match self.peek().token_type.clone() {
            TokenType::Identifier(id) => {
                self.advance();
                id
            }
            _ => {
                return Err(format!(
                    "Nom de variable attendu après 'Jib' à la ligne {}, colonne {}",
                    self.peek().span.line,
                    self.peek().span.col
                ))
            }
        };
        Ok(Stmt::Input { name, span })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let span = self.previous().span;
        let condition = self.expression()?;
        let mut then_branch = Vec::new();
        let mut else_branch = None;

        while !self.is_at_end() && !self.check(&TokenType::Walou) && !self.check(&TokenType::Khalas) {
            then_branch.push(self.statement()?);
        }

        if self.match_token(&[TokenType::Walou]) {
            let mut else_stmts = Vec::new();
            while !self.is_at_end() && !self.check(&TokenType::Khalas) {
                else_stmts.push(self.statement()?);
            }
            else_branch = Some(else_stmts);
        }

        self.consume(&TokenType::Khalas, "Mot-clé 'Khalas' attendu pour fermer la condition 'Zaama'")?;

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        let span = self.previous().span;
        let condition = self.expression()?;
        let mut body = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::Baraka) {
            body.push(self.statement()?);
        }

        self.consume(&TokenType::Baraka, "Mot-clé 'Baraka' attendu pour fermer la boucle 'Dima'")?;

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        // Affectation x = expr
        if let Expr::Variable(name, span) = &expr {
            if self.match_token(&[TokenType::Equal]) {
                let value = self.expression()?;
                return Ok(Stmt::Assign {
                    name: name.clone(),
                    value,
                    span: *span,
                });
            }
        }

        let span = expr.span();
        Ok(Stmt::ExprStmt { expr, span })
    }

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.previous().token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let span = self.previous().span;
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = match self.previous().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let span = self.previous().span;
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let span = self.previous().span;
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Modulo]) {
            let op = match self.previous().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Modulo => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let span = self.previous().span;
            let right = self.primary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.advance();

        match token.token_type {
            TokenType::Number(n) => Ok(Expr::Literal(Value::Number(n), token.span)),
            TokenType::String(s) => Ok(Expr::Literal(Value::String(s), token.span)),
            TokenType::Sah => Ok(Expr::Literal(Value::Bool(true), token.span)),
            TokenType::K7hba => Ok(Expr::Literal(Value::Bool(false), token.span)),
            TokenType::Identifier(id) => Ok(Expr::Variable(id, token.span)),
            TokenType::Jib => Ok(Expr::Input(token.span)),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Parenthèse fermante ')' attendue")?;
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                    span: token.span,
                })
            }
            _ => Err(format!(
                "Expression attendue à la ligne {}, colonne {} (token trouvé: {:?})",
                token.span.line, token.span.col, token.token_type
            )),
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!(
                "{} (à la ligne {}, colonne {})",
                message,
                self.peek().span.line,
                self.peek().span.col
            ))
        }
    }
}
