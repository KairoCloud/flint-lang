use crate::ast::*;
use crate::lexer::{SpannedToken, Token, Lexer};
use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    current: Option<SpannedToken>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        let lexer = Lexer::new(input).peekable();
        let mut parser = Parser { lexer, current: None };
        parser.advance();
        parser
    }

    fn advance(&mut self) {
        self.current = self.lexer.next();
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.current.as_ref()
    }

    fn expect(&mut self, expected: Token) -> Result<SpannedToken, String> {
        if let Some(ref t) = self.current {
            if std::mem::discriminant(&t.token) == std::mem::discriminant(&expected) {
                let t = self.current.take().unwrap();
                self.advance();
                return Ok(t);
            }
        }
        Err(format!("expected {:?}, found {:?}", expected, self.current.as_ref().map(|t| &t.token)))
    }

    fn match_token(&mut self, expected: Token) -> bool {
        if let Some(ref t) = self.current {
            std::mem::discriminant(&t.token) == std::mem::discriminant(&expected)
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(Token::Newline) {
            self.advance();
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.match_token(Token::Eof) {
            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => return Err(e),
            }
            self.skip_newlines();
        }
        Ok(Program::new(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek().map(|t| &t.token) {
            Some(Token::Let) => self.parse_let(),
            Some(Token::Var) => self.parse_var(),
            Some(Token::Const) => self.parse_const(),
            Some(Token::Fn) => self.parse_function(),
            Some(Token::Struct) => self.parse_struct(),
            Some(Token::Enum) => self.parse_enum(),
            Some(Token::Interface) => self.parse_interface(),
            Some(Token::Trait) => self.parse_trait(),
            Some(Token::Impl) => self.parse_impl(),
            Some(Token::Extend) => self.parse_extend(),
            Some(Token::Import) => self.parse_import(),
            Some(Token::Export) => self.parse_export(),
            Some(Token::Test) => self.parse_test(),
            Some(Token::Ai) => self.parse_ai_block(),
            Some(Token::Assert) => self.parse_assert(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => self.parse_break(),
            Some(Token::Continue) => self.parse_continue(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'let'
        let pat = self.parse_pat()?;
        let ty = if self.match_token(Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.match_token(Token::Not) {
            self.advance();
            None
        } else if self.match_token(Token::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Stmt::Let { pat, ty, init })
    }

    fn parse_var(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'var'
        let pat = self.parse_pat()?;
        let ty = if self.match_token(Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.match_token(Token::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Stmt::Var { pat, ty, init })
    }

    fn parse_const(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'const'
        let pat = self.parse_pat()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let init = self.parse_expr()?;
        Ok(Stmt::Const { pat, ty, init })
    }

    fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'fn'
        let is_async = false;
        let is_pub = false;

        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected function name".to_string()),
        };

        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while !self.match_token(Token::RParen) {
            params.push(self.parse_param()?);
            if self.match_token(Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RParen)?;

        let return_type = if self.match_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.match_token(Token::Colon) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(Stmt::Function(Function {
            name, params, return_type, body, is_async, is_pub,
        }))
    }

    fn parse_param(&mut self) -> Result<Param, String> {
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected parameter name".to_string()),
        };

        let ty = if self.match_token(Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let default = if self.match_token(Token::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Param { name, ty, default, is_mut: false })
    }

    fn parse_struct(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'struct'
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected struct name".to_string()),
        };

        self.expect(Token::Colon)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            if self.match_token(Token::Fn) {
                methods.push(self.parse_method()?);
            } else {
                fields.push(self.parse_field()?);
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Struct(StructDecl {
            name,
            fields,
            methods,
            is_pub: false,
        }))
    }

    fn parse_field(&mut self) -> Result<Field, String> {
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected field name".to_string()),
        };

        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;

        Ok(Field {
            name,
            ty,
            is_pub: false,
            is_mut: false,
        })
    }

    fn parse_method(&mut self) -> Result<Function, String> {
        self.advance(); // consume 'fn'
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected method name".to_string()),
        };

        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while !self.match_token(Token::RParen) {
            params.push(self.parse_param()?);
            if self.match_token(Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RParen)?;

        let return_type = if self.match_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.match_token(Token::Colon) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_async: false,
            is_pub: false,
        })
    }

    fn parse_enum(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'enum'
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected enum name".to_string()),
        };

        self.expect(Token::Colon)?;

        let mut variants = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            let var_name = match self.peek().map(|t| &t.token) {
                Some(Token::Ident(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => return Err("expected variant name".to_string()),
            };

            let mut fields = Vec::new();
            if self.match_token(Token::LParen) {
                self.advance();
                while !self.match_token(Token::RParen) {
                    fields.push(self.parse_field()?);
                    if self.match_token(Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
            }

            variants.push(Variant { name: var_name, fields });
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Enum(EnumDecl {
            name,
            variants,
            is_pub: false,
        }))
    }

    fn parse_interface(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'interface'
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected interface name".to_string()),
        };

        self.expect(Token::Colon)?;

        let mut methods = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            methods.push(self.parse_method()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Interface(InterfaceDecl {
            name,
            methods,
            is_pub: false,
        }))
    }

    fn parse_trait(&mut self) -> Result<Stmt, String> {
        self.advance();
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected trait name".to_string()),
        };

        self.expect(Token::Colon)?;

        let mut methods = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            methods.push(self.parse_method()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Trait(TraitDecl {
            name,
            methods,
            is_pub: false,
        }))
    }

    fn parse_impl(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'impl'
        let ty = self.parse_type()?;

        self.expect(Token::Colon)?;

        let mut methods = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            methods.push(self.parse_method()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Impl(ImplBlock { ty, methods }))
    }

    fn parse_extend(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'extend'
        let ty = self.parse_type()?;

        self.expect(Token::Colon)?;

        let mut methods = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            methods.push(self.parse_method()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Stmt::Impl(ImplBlock { ty, methods }))
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'import'
        
        let path = match self.peek().map(|t| &t.token) {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err("expected import path".to_string()),
        };

        let items = if self.match_token(Token::From) {
            self.advance();
            let mut items = Vec::new();
            while !self.match_token(Token::Eof) && !self.match_token(Token::Newline) {
                match self.peek().map(|t| &t.token) {
                    Some(Token::Ident(n)) => {
                        let name = n.clone();
                        self.advance();
                        let alias = if self.match_token(Token::As) {
                            self.advance();
                            Some(match self.peek().map(|t| &t.token) {
                                Some(Token::Ident(a)) => {
                                    let a = a.clone();
                                    self.advance();
                                    a
                                }
                                _ => return Err("expected alias".to_string()),
                            })
                        } else {
                            None
                        };
                        items.push((name, alias));
                    }
                    Some(Token::LBrace) => {
                        self.advance();
                        while !self.match_token(Token::RBrace) {
                            match self.peek().map(|t| &t.token) {
                                Some(Token::Ident(n)) => {
                                    let name = n.clone();
                                    self.advance();
                                    let alias = if self.match_token(Token::As) {
                                        self.advance();
                                        Some(match self.peek().map(|t| &t.token) {
                                            Some(Token::Ident(a)) => {
                                                let a = a.clone();
                                                self.advance();
                                                a
                                            }
                                            _ => return Err("expected alias".to_string()),
                                        })
                                    } else {
                                        None
                                    };
                                    items.push((name, alias));
                                }
                                _ => break,
                            }
                            if self.match_token(Token::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(Token::RBrace)?;
                    }
                    _ => break,
                }
                if self.match_token(Token::Comma) {
                    self.advance();
                }
            }
            items
        } else {
            Vec::new()
        };

        Ok(Stmt::Import(ImportDecl {
            path,
            items,
            is_pub: false,
        }))
    }

    fn parse_export(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'export'
        let mut items = Vec::new();
        while !self.match_token(Token::Eof) && !self.match_token(Token::Newline) {
            match self.peek().map(|t| &t.token) {
                Some(Token::Ident(n)) => {
                    items.push(n.clone());
                    self.advance();
                }
                _ => break,
            }
            if self.match_token(Token::Comma) {
                self.advance();
            }
        }
        Ok(Stmt::Export(ExportDecl { items }))
    }

    fn parse_test(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'test'
        let name = match self.peek().map(|t| &t.token) {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected test name".to_string()),
        };

        self.expect(Token::Colon)?;
        
        let mut body = Vec::new();
        while !self.match_token(Token::Eof) && !self.match_token(Token::RBrace) {
            body.push(self.parse_stmt()?);
        }
        if self.match_token(Token::RBrace) {
            self.advance();
        }

        Ok(Stmt::Test { name, body })
    }

    fn parse_ai_block(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'ai'
        self.expect(Token::Colon)?;

        let mut stmts = Vec::new();
        while !self.match_token(Token::Eof) && !self.match_token(Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        if self.match_token(Token::RBrace) {
            self.advance();
        }

        Ok(Stmt::AiBlock { stmts })
    }

    fn parse_assert(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'assert'
        let cond = self.parse_expr()?;
        let msg = if self.match_token(Token::Comma) {
            self.advance();
            match self.peek().map(|t| &t.token) {
                Some(Token::String(s)) => {
                    let s = s.clone();
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };
        Ok(Stmt::Assert { cond, msg })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'
        if self.match_token(Token::Eof) || self.match_token(Token::Newline) {
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_break(&mut self) -> Result<Stmt, String> {
        self.advance();
        if self.match_token(Token::Eof) || self.match_token(Token::Newline) {
            return Ok(Stmt::Break(None));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Break(Some(expr)))
    }

    fn parse_continue(&mut self) -> Result<Stmt, String> {
        self.advance();
        Ok(Stmt::Continue)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and_expr()?;
        while self.match_token(Token::OrOr) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality_expr()?;
        while self.match_token(Token::AndAnd) {
            self.advance();
            let right = self.parse_equality_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison_expr()?;
        while self.match_token(Token::Eq) || self.match_token(Token::NotEq) {
            let op = if self.match_token(Token::Eq) {
                self.advance();
                BinOp::Eq
            } else {
                self.advance();
                BinOp::Neq
            };
            let right = self.parse_comparison_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_range_expr()?;
        while matches!(self.peek().map(|t| &t.token), 
            Some(Token::Lt) | Some(Token::LtEq) | Some(Token::Gt) | Some(Token::GtEq) | Token::In | Token::Is) {
            let op = match self.peek().map(|t| &t.token) {
                Some(Token::Lt) => { self.advance(); BinOp::Lt }
                Some(Token::LtEq) => { self.advance(); BinOp::LtEq }
                Some(Token::Gt) => { self.advance(); BinOp::Gt }
                Some(Token::GtEq) => { self.advance(); BinOp::GtEq }
                Some(Token::In) => { self.advance(); BinOp::In }
                Some(Token::Is) => { self.advance(); BinOp::Is }
                _ => break,
            };
            let right = self.parse_range_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_range_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_add_expr()?;
        while self.match_token(Token::DotDot) || self.match_token(Token::DotDotDot) {
            self.advance();
            let right = self.parse_add_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Range,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul_expr()?;
        while matches!(self.peek().map(|t| &t.token), Some(Token::Plus) | Some(Token::Minus) | Token::Pipe) {
            let op = match self.peek().map(|t| &t.token) {
                Some(Token::Plus) => { self.advance(); BinOp::Add }
                Some(Token::Minus) => { self.advance(); BinOp::Sub }
                Some(Token::Pipe) => { self.advance(); BinOp::BitOr }
                _ => break,
            };
            let right = self.parse_mul_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary_expr()?;
        while matches!(self.peek().map(|t| &t.token), 
            Some(Token::Star) | Some(Token::Slash) | Some(Token::Percent) | Token::StarStar | Token::Ampersand) {
            let op = match self.peek().map(|t| &t.token) {
                Some(Token::Star) => { self.advance(); BinOp::Mul }
                Some(Token::Slash) => { self.advance(); BinOp::Div }
                Some(Token::Percent) => { self.advance(); BinOp::Mod }
                Some(Token::StarStar) => { self.advance(); BinOp::Pow }
                Some(Token::Ampersand) => { self.advance(); BinOp::BitAnd }
                _ => break,
            };
            let right = self.parse_unary_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, String> {
        if let Some(tok) = self.peek() {
            match &tok.token {
                Token::Not => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    return Ok(Expr::Unary { op: UnOp::Not, expr: Box::new(expr) });
                }
                Token::Minus => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    return Ok(Expr::Unary { op: UnOp::Neg, expr: Box::new(expr) });
                }
                Token::Await => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    return Ok(Expr::Await(Box::new(expr)));
                }
                _ => {}
            }
        }
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.match_token(Token::LParen) {
                self.advance();
                let mut args = Vec::new();
                while !self.match_token(Token::RParen) {
                    args.push(self.parse_arg()?);
                    if self.match_token(Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                expr = Expr::Call { func: Box::new(expr), args };
            } else if self.match_token(Token::Dot) {
                self.advance();
                let method = match self.peek().map(|t| &t.token) {
                    Some(Token::Ident(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => return Err("expected method name".to_string()),
                };
                let args = if self.match_token(Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.match_token(Token::RParen) {
                        args.push(self.parse_arg()?);
                        if self.match_token(Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    args
                } else {
                    Vec::new()
                };
                expr = Expr::MethodChain {
                    base: Box::new(expr),
                    calls: vec![MethodCall { name: method, args }],
                };
            } else if self.match_token(Token::LBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index { array: Box::new(expr), index: Box::new(index) };
            } else if self.match_token(Token::QuestionDot) {
                self.advance();
                let method = match self.peek().map(|t| &t.token) {
                    Some(Token::Ident(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => return Err("expected method name".to_string()),
                };
                let args = if self.match_token(Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.match_token(Token::RParen) {
                        args.push(self.parse_arg()?);
                        if self.match_token(Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    args
                } else {
                    Vec::new()
                };
                expr = Expr::MethodChain {
                    base: Box::new(expr),
                    calls: vec![MethodCall { name: method, args }],
                };
            } else if self.match_token(Token::DoubleQuestion) {
                self.advance();
                let right = self.parse_expr()?;
                expr = Expr::OptionCoalesce {
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arg(&mut self) -> Result<Arg, String> {
        let name = if matches!(self.peek().map(|t| &t.token), Some(Token::Ident(_))) {
            let name = match self.peek().map(|t| &t.token) {
                Some(Token::Ident(n)) => Some(n.clone()),
                _ => None,
            };
            self.advance();
            if self.match_token(Token::Colon) {
                self.advance();
                name
            } else {
                None
            }
        } else {
            None
        };
        let expr = self.parse_expr()?;
        Ok(Arg { name, expr })
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, String> {
        match self.peek().map(|t| &t.token) {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Float(n)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(Token::MultilineString(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::MultilineString(s)))
            }
            Some(Token::Char(c)) => {
                let c = *c;
                self.advance();
                Ok(Expr::Literal(Literal::Char(c)))
            }
            Some(Token::LParen) => {
                self.advance();
                if self.match_token(Token::RParen) {
                    self.advance();
                    return Ok(Expr::Tuple(Vec::new()));
                }
                let expr = self.parse_expr()?;
                if self.match_token(Token::Comma) {
                    self.advance();
                    let mut exprs = vec![expr];
                    while !self.match_token(Token::RParen) {
                        exprs.push(self.parse_expr()?);
                        if self.match_token(Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    return Ok(Expr::Tuple(exprs));
                }
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBracket) => {
                self.advance();
                let mut exprs = Vec::new();
                while !self.match_token(Token::RBracket) {
                    exprs.push(self.parse_expr()?);
                    if self.match_token(Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::Array(exprs))
            }
            Some(Token::LBrace) => {
                self.advance();
                let mut exprs = Vec::new();
                while !self.match_token(Token::RBrace) {
                    exprs.push(self.parse_expr()?);
                    if self.match_token(Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr::Block(exprs))
            }
            Some(Token::If) => self.parse_if(),
            Some(Token::Match) => self.parse_match(),
            Some(Token::For) => self.parse_for(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Fn) => self.parse_lambda(),
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                Ok(Expr::Ident(n))
            }
            Some(Token::Channel) => {
                self.advance();
                let ty = if self.match_token(Token::Lt) {
                    self.advance();
                    let ty = self.parse_type()?;
                    self.expect(Token::Gt)?;
                    Some(ty)
                } else {
                    None
                };
                Ok(Expr::Channel(ty.unwrap_or(Type::Unit)))
            }
            _ => Err(format!("unexpected token: {:?}", self.current)),
        }
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'if'
        let condition = self.parse_expr()?;
        self.expect(Token::Colon)?;
        let then_branch = Box::new(self.parse_expr()?);

        let mut elif_branches = Vec::new();
        while self.match_token(Token::Elif) {
            self.advance();
            let elif_cond = self.parse_expr()?;
            self.expect(Token::Colon)?;
            let elif_body = self.parse_expr()?;
            elif_branches.push(Elif { condition: elif_cond, body: elif_body });
        }

        let else_branch = if self.match_token(Token::Else) {
            self.advance();
            if self.match_token(Token::Colon) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch,
            elif_branches,
            else_branch,
        })
    }

    fn parse_match(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'match'
        let expr = self.parse_expr()?;
        self.expect(Token::Colon)?;

        let mut arms = Vec::new();
        while !self.match_token(Token::RBrace) && !self.match_token(Token::Eof) {
            let pattern = self.parse_pat()?;
            let guard = if self.match_token(Token::If) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(Token::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(MatchArm { pattern, guard, body });
        }

        if self.match_token(Token::RBrace) {
            self.advance();
        }

        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn parse_for(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'for'
        let var = match self.peek().map(|t| &t.token) {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err("expected loop variable".to_string()),
        };
        self.expect(Token::In)?;
        let iter = self.parse_expr()?;
        self.expect(Token::Colon)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Expr::For { var, iter, body })
    }

    fn parse_while(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'while'
        let condition = self.parse_expr()?;
        self.expect(Token::Colon)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Expr::While { condition, body })
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'fn'
        
        let params = if self.match_token(Token::LParen) {
            self.advance();
            let mut params = Vec::new();
            while !self.match_token(Token::RParen) {
                params.push(self.parse_param()?);
                if self.match_token(Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            params
        } else if let Some(Token::Ident(n)) = self.peek().map(|t| &t.token).cloned() {
            self.advance();
            vec![Param { name: n, ty: None, default: None, is_mut: false }]
        } else {
            Vec::new()
        };

        let return_type = if self.match_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::FatArrow)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Expr::Lambda { params, body })
    }

    fn parse_pat(&mut self) -> Result<Pat, String> {
        match self.peek().map(|t| &t.token) {
            Some(Token::Underscore) => {
                self.advance();
                Ok(Pat::Wildcard)
            }
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                Ok(Pat::Ident(n))
            }
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Ok(Pat::Literal(Literal::Int(n)))
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                Ok(Pat::Literal(Literal::Float(n)))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Pat::Literal(Literal::String(s)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Pat::Literal(Literal::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Pat::Literal(Literal::Bool(false)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Pat::Literal(Literal::Null))
            }
            Some(Token::LParen) => {
                self.advance();
                if self.match_token(Token::RParen) {
                    self.advance();
                    return Ok(Pat::Tuple(Vec::new()));
                }
                let mut pats = vec![self.parse_pat()?];
                while self.match_token(Token::Comma) {
                    self.advance();
                    pats.push(self.parse_pat()?);
                }
                self.expect(Token::RParen)?;
                Ok(Pat::Tuple(pats))
            }
            Some(Token::LBracket) => {
                self.advance();
                let mut pats = Vec::new();
                let rest = if self.match_token(Token::DotDotDot) {
                    self.advance();
                    true
                } else {
                    while !self.match_token(Token::RBracket) {
                        pats.push(self.parse_pat()?);
                        if self.match_token(Token::Comma) {
                            self.advance();
                        }
                    }
                    false
                };
                self.expect(Token::RBracket)?;
                Ok(Pat::Array(pats, rest))
            }
            _ => Err(format!("unexpected pattern: {:?}", self.current)),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().map(|t| &t.token) {
            Some(Token::Int(_)) => { self.advance(); Ok(Type::Int) }
            Some(Token::Float(_)) => { self.advance(); Ok(Type::Float) }
            Some(Token::Bool(_)) => { self.advance(); Ok(Type::Bool) }
            Some(Token::Str(_)) => { self.advance(); Ok(Type::Str) }
            Some(Token::Char(_)) => { self.advance(); Ok(Type::Char) }
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                
                if self.match_token(Token::Lt) {
                    self.advance();
                    let mut args = vec![self.parse_type()?];
                    while self.match_token(Token::Comma) {
                        self.advance();
                        args.push(self.parse_type()?);
                    }
                    self.expect(Token::Gt)?;
                    return Ok(Type::Generic(n, args));
                }
                
                Ok(Type::Custom(n))
            }
            Some(Token::LBracket) => {
                self.advance();
                let inner = Box::new(self.parse_type()?);
                self.expect(Token::RBracket)?;
                Ok(Type::Array(inner))
            }
            Some(Token::Question) => {
                self.advance();
                let inner = Box::new(self.parse_type()?);
                Ok(Type::Nullable(inner))
            }
            Some(Token::LParen) => {
                self.advance();
                let mut types = Vec::new();
                while !self.match_token(Token::RParen) {
                    types.push(self.parse_type()?);
                    if self.match_token(Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Type::Tuple(types))
            }
            _ => Ok(Type::Unit),
        }
    }
}

pub fn parse(input: &str) -> Result<Program, String> {
    Parser::new(input).parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_stmt() {
        let result = parse("let x = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_var_stmt() {
        let result = parse("var count: Int = 0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_const_stmt() {
        let result = parse("const MAX: Int = 100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function() {
        let result = parse("fn add(a: Int, b: Int) -> Int: a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct() {
        let input = "struct Point:
  x: Float
  y: Float";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum() {
        let input = "enum Result:
  Ok(value: Int)
  Err(error: Str)";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_expr() {
        let result = parse("if x > 0: true else: false");
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_expr() {
        let result = parse("match x: 0 => \"zero\" _ => \"other\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda() {
        let result = parse("let add = (a, b) => a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_ops() {
        let result = parse("let x = 1 + 2 * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex() {
        let input = r#"
let name = "Flint"
var count: Int = 0

fn greet(name: Str) -> Str:
  return "Hello, ${name}!"

if count > 0:
  print("positive")
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }
}