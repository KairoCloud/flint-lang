use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let, Var, Fn, Return, If, Else, Elif, For, While, Match,
    In, Is, Not, And, Or, True, False, Null,
    Type, Interface, Trait, Impl, Enum, Struct,
    Import, Export, From, As,
    Async, Await, Spawn, Channel, Send, Recv,
    Pub, Priv, Static, Const, Mut, Self_, Super,
    Use, Mod, Test, Assert, Ai, Prompt, Agent,
    Extend, Where, AsyncBlock, SpawnBlock,

    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    MultilineString(String),

    // Operators
    Plus, Minus, Star, Slash, Percent, StarStar,
    Eq, NotEq, Lt, LtEq, Gt, GtEq,
    AndAnd, OrOr, Not,
    PlusEq, MinusEq, StarEq, SlashEq,
    Arrow, FatArrow,
    Pipe, Ampersand, Question, QuestionDot, DoubleQuestion,
    Colon, DoubleColon, DotDot, DotDotDot, Dot,
    At,

    // Punctuation
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Semicolon, Hash, Period,

    // Indentation
    Indent, Dedent, Newline,

    // Special
    Eof,
    Error(String),
    Ident(String),
}

impl Token {
    pub fn is_keyword(s: &str) -> Option<Token> {
        match s {
            "let" => Some(Token::Let),
            "var" => Some(Token::Var),
            "fn" => Some(Token::Fn),
            "return" => Some(Token::Return),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "elif" => Some(Token::Elif),
            "for" => Some(Token::For),
            "while" => Some(Token::While),
            "match" => Some(Token::Match),
            "in" => Some(Token::In),
            "is" => Some(Token::Is),
            "not" => Some(Token::Not),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            "null" => Some(Token::Null),
            "type" => Some(Token::Type),
            "interface" => Some(Token::Interface),
            "trait" => Some(Token::Trait),
            "impl" => Some(Token::Impl),
            "enum" => Some(Token::Enum),
            "struct" => Some(Token::Struct),
            "import" => Some(Token::Import),
            "export" => Some(Token::Export),
            "from" => Some(Token::From),
            "as" => Some(Token::As),
            "async" => Some(Token::Async),
            "await" => Some(Token::Await),
            "spawn" => Some(Token::Spawn),
            "channel" => Some(Token::Channel),
            "send" => Some(Token::Send),
            "recv" => Some(Token::Recv),
            "pub" => Some(Token::Pub),
            "priv" => Some(Token::Priv),
            "static" => Some(Token::Static),
            "const" => Some(Token::Const),
            "mut" => Some(Token::Mut),
            "self" => Some(Token::Self_),
            "super" => Some(Token::Super),
            "use" => Some(Token::Use),
            "mod" => Some(Token::Mod),
            "test" => Some(Token::Test),
            "assert" => Some(Token::Assert),
            "ai" => Some(Token::Ai),
            "prompt" => Some(Token::Prompt),
            "agent" => Some(Token::Agent),
            "extend" => Some(Token::Extend),
            "where" => Some(Token::Where),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    fn new(line: usize, column: usize, offset: usize) -> Self {
        Position { line, column, offset }
    }
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    pos: Position,
    peeked: Vec<SpannedToken>,
    indent_stack: Vec<usize>,
    had_nl: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input: input.chars().peekable(),
            pos: Position::new(1, 0, 0),
            peeked: Vec::new(),
            indent_stack: vec![0],
            had_nl: false,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input.next()?;
        let new_offset = self.pos.offset + 1;
        let (new_line, new_column) = if ch == '\n' {
            (self.pos.line + 1, 0)
        } else {
            (self.pos.line, self.pos.column + 1)
        };
        self.pos = Position::new(new_line, new_column, new_offset);
        Some(ch)
    }

    fn skip_whitespace(&mut self) -> Option<char> {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() && ch != '\n' {
                self.next_char();
            } else {
                break;
            }
        }
        self.peek_char()
    }

    fn make_token(&self, token: Token) -> SpannedToken {
        SpannedToken {
            token,
            span: Span {
                start: self.pos,
                end: self.pos,
            },
        }
    }

    fn make_token_with_end(&self, token: Token, end: Position) -> SpannedToken {
        SpannedToken {
            token,
            span: Span {
                start: self.pos,
                end,
            },
        }
    }

    pub fn next_token(&mut self) -> SpannedToken {
        if let Some(t) = self.peeked.pop() {
            return t;
        }
        self.do_next_token()
    }

    fn do_next_token(&mut self) -> SpannedToken {
        self.skip_whitespace();

        let ch = match self.peek_char() {
            None => return self.make_token(Token::Eof),
            Some(ch) => ch,
        };

        // Handle newlines specially
        if ch == '\n' {
            self.next_char();
            self.had_nl = true;
            return self.make_token(Token::Newline);
        }

        // Comments
        if ch == '#' {
            return self.read_comment();
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_ident();
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // Strings
        if ch == '"' || ch == '\'' {
            return self.read_string(ch);
        }

        // Multi-character operators
        match ch {
            '/' => self.read_slash(),
            '-' => self.read_arrow_or_minus(),
            '=' => self.read_equals(),
            '!' => self.read_bang(),
            '<' => self.read_lt(),
            '>' => self.read_gt(),
            '&' => self.read_ampersand(),
            '|' => self.read_pipe(),
            ':' => self.read_colon(),
            '.' => self.read_dot(),
            '?' => self.read_question(),
            '*' => self.read_star(),
            '+' => self.read_plus(),
            _ => self.read_single_char(ch),
        }
    }

    fn read_comment(&mut self) -> SpannedToken {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.next_char();
        }
        self.make_token_with_end(Token::Ident("comment".to_string()), start)
    }

    fn read_ident(&mut self) -> SpannedToken {
        let start = self.pos;
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        let end = self.pos;
        let token = if let Some(kw) = Token::is_keyword(&s) {
            kw
        } else {
            Token::Ident(s)
        };
        SpannedToken { token, span: Span { start, end } }
    }

    fn read_number(&mut self) -> SpannedToken {
        let start = self.pos;
        let mut s = String::new();
        let mut is_float = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                s.push(self.next_char().unwrap());
            } else if ch == '.' && !is_float {
                let next = self.input.clone().nth(1);
                if let Some(n) = next, n.is_ascii_digit() {
                    is_float = true;
                    s.push(self.next_char().unwrap());
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let end = self.pos;
        let token = if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Int(s.parse().unwrap_or(0))
        };

        SpannedToken { token, span: Span { start, end } }
    }

    fn read_string(&mut self, quote: char) -> SpannedToken {
        let start = self.pos;
        self.next_char(); // consume opening quote

        // Check for triple-quoted string
        let is_triple = self.peek_char() == Some(quote) 
            && self.input.clone().nth(1) == Some(quote);

        if is_triple {
            self.next_char();
            self.next_char();
            return self.read_multiline_string(quote);
        }

        let mut s = String::new();
        let mut is_interpolated = false;

        while let Some(ch) = self.peek_char() {
            if ch == quote {
                self.next_char();
                let end = self.pos;
                let tok = if is_interpolated {
                    Token::String(s)
                } else {
                    Token::String(s)
                };
                return SpannedToken { token: tok, span: Span { start, end } };
            }
            if ch == '\\' {
                self.next_char();
                if let Some(esc) = self.read_escape() {
                    s.push(esc);
                }
                continue;
            }
            if ch == '$' && self.input.clone().nth(1) == Some('{') {
                is_interpolated = true;
            }
            s.push(self.next_char().unwrap());
        }

        self.make_token(Token::Error("unterminated string".to_string()))
    }

    fn read_multiline_string(&mut self, quote: char) -> SpannedToken {
        let start = self.pos;
        let mut s = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == quote {
                let next1 = self.input.clone().nth(1);
                let next2 = self.input.clone().nth(2);
                if next1 == Some(quote) && next2 == Some(quote) {
                    self.next_char();
                    self.next_char();
                    self.next_char();
                    let end = self.pos;
                    return SpannedToken {
                        token: Token::MultilineString(s),
                        span: Span { start, end },
                    };
                }
            }
            s.push(self.next_char().unwrap());
        }

        self.make_token(Token::Error("unterminated multiline string".to_string()))
    }

    fn read_escape(&mut self) -> Option<char> {
        match self.peek_char() {
            Some('n') => { self.next_char(); Some('\n') }
            Some('t') => { self.next_char(); Some('\t') }
            Some('r') => { self.next_char(); Some('\r') }
            Some('\\') => { self.next_char(); Some('\\') }
            Some('"') => { self.next_char(); Some('"') }
            Some('\'') => { self.next_char(); Some('\'') }
            Some('$') => { self.next_char(); Some('$') }
            _ => None,
        }
    }

    fn read_slash(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::SlashEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Slash, self.pos),
        }
    }

    fn read_arrow_or_minus(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('>') => {
                self.next_char();
                self.make_token_with_end(Token::Arrow, self.pos)
            }
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::MinusEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Minus, self.pos),
        }
    }

    fn read_equals(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::Eq, self.pos)
            }
            Some('>') => {
                self.next_char();
                self.make_token_with_end(Token::FatArrow, self.pos)
            }
            _ => self.make_token_with_end(Token::Not, self.pos), // = is assignment
        }
    }

    fn read_bang(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::NotEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Not, self.pos),
        }
    }

    fn read_lt(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::LtEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Lt, self.pos),
        }
    }

    fn read_gt(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::GtEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Gt, self.pos),
        }
    }

    fn read_ampersand(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('&') => {
                self.next_char();
                self.make_token_with_end(Token::AndAnd, self.pos)
            }
            _ => self.make_token_with_end(Token::Ampersand, self.pos),
        }
    }

    fn read_pipe(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('|') => {
                self.next_char();
                self.make_token_with_end(Token::OrOr, self.pos)
            }
            _ => self.make_token_with_end(Token::Pipe, self.pos),
        }
    }

    fn read_colon(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some(':') => {
                self.next_char();
                self.make_token_with_end(Token::DoubleColon, self.pos)
            }
            _ => self.make_token_with_end(Token::Colon, self.pos),
        }
    }

    fn read_dot(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('.') => {
                let next = self.input.clone().nth(1);
                if next == Some('.') {
                    self.next_char();
                    self.next_char();
                    return self.make_token_with_end(Token::DotDotDot, self.pos);
                }
                self.next_char();
                self.make_token_with_end(Token::DotDot, self.pos)
            }
            _ => self.make_token_with_end(Token::Dot, self.pos),
        }
    }

    fn read_question(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('.') => {
                self.next_char();
                self.make_token_with_end(Token::QuestionDot, self.pos)
            }
            Some('?') => {
                self.next_char();
                self.make_token_with_end(Token::DoubleQuestion, self.pos)
            }
            _ => self.make_token_with_end(Token::Question, self.pos),
        }
    }

    fn read_star(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('*') => {
                self.next_char();
                self.make_token_with_end(Token::StarStar, self.pos)
            }
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::StarEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Star, self.pos),
        }
    }

    fn read_plus(&mut self) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        match self.peek_char() {
            Some('=') => {
                self.next_char();
                self.make_token_with_end(Token::PlusEq, self.pos)
            }
            _ => self.make_token_with_end(Token::Plus, self.pos),
        }
    }

    fn read_single_char(&mut self, ch: char) -> SpannedToken {
        let start = self.pos;
        self.next_char();
        let token = match ch {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '@' => Token::At,
            _ => Token::Error(format!("unexpected character: {}", ch)),
        };
        self.make_token_with_end(token, self.pos)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.token == Token::Eof {
            None
        } else {
            Some(token)
        }
    }
}

pub fn tokenize(input: &str) -> Vec<SpannedToken> {
    Lexer::new(input).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(s: &str) -> Token {
        Token::Ident(s.to_string())
    }

    fn kw(s: &str) -> Token {
        Token::is_keyword(s).unwrap()
    }

    #[test]
    fn test_keywords() {
        let input = "let var fn return if else elif for while match in is not and or true false null";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 21);
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Var);
        assert_eq!(tokens[2].token, Token::Fn);
    }

    #[test]
    fn test_identifiers() {
        let input = "foo bar _private myVar123";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token, tok("foo"));
        assert_eq!(tokens[1].token, tok("bar"));
        assert_eq!(tokens[2].token, tok("_private"));
        assert_eq!(tokens[3].token, tok("myVar123"));
    }

    #[test]
    fn test_integers() {
        let input = "42 0 123456789";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Int(0));
        assert_eq!(tokens[2].token, Token::Int(123456789));
    }

    #[test]
    fn test_floats() {
        let input = "3.14 0.5 42.0";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Float(3.14));
        assert_eq!(tokens[1].token, Token::Float(0.5));
    }

    #[test]
    fn test_strings() {
        let input = "\"hello\" 'world'";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::String("hello".to_string()));
        assert_eq!(tokens[1].token, Token::String("world".to_string()));
    }

    #[test]
    fn test_multiline_strings() {
        let input = "\"\"\"hello\nworld\"\"\"";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::MultilineString("hello\nworld".to_string()));
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / % ** == != < > <= >= && || ! = += -= *= /= -> => | & ? ?? ?.";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 24);
        assert_eq!(tokens[0].token, Token::Plus);
        assert_eq!(tokens[1].token, Token::Minus);
        assert_eq!(tokens[2].token, Token::Star);
        assert_eq!(tokens[18].token, Token::AndAnd);
        assert_eq!(tokens[19].token, Token::OrOr);
    }

    #[test]
    fn test_punctuation() {
        let input = "( ) { } [ ] , . ; #";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::RParen);
    }

    #[test]
    fn test_colon_operators() {
        let input = ": ::";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Colon);
        assert_eq!(tokens[1].token, Token::DoubleColon);
    }

    #[test]
    fn test_dot_operators() {
        let input = ". .. ...";
        let tokens: Vec<_> = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Dot);
        assert_eq!(tokens[1].token, Token::DotDot);
        assert_eq!(tokens[2].token, Token::DotDotDot);
    }

    #[test]
    fn test_full_example() {
        let input = r#"
let name = "Flint"
var count: Int = 0
const MAX: Int = 100

fn greet(name: Str) -> Str:
  return "Hello, ${name}!"

if count > 0:
  print("positive")
"#;
        let tokens: Vec<_> = tokenize(input);
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_type_keywords() {
        let input = "type interface trait impl enum struct import export from as async await spawn channel send recv pub priv static const mut self super use mod test assert ai prompt agent extend where";
        let tokens: Vec<_> = tokenize(input);
        assert!(tokens.len() > 0);
    }
}