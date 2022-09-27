use boa_unicode::UnicodeProperties;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    String,
    NonTerminatedString,
    HexLit,
    BinLit,
    NumLit,
    InvalidNonTerminatedComment,
    InvalidNewlineString,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    Bang,
    Caret,
    Asterisk,
    Amp,
    And,
    Pipe,
    Or,
    Plus,
	Increment,
    AddAssign,
    Minus,
	Decrement,
    SubAssign,
    Div,
    DivAssign,
    Equal,
    EqualEqual,
    EqualEqualEqual,
    LessOrEqual,
    GreaterOrEqual,
    FatArrow,
    Ident,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Token<'a> {
	pub index: u32,
	pub end_index: u32,
    pub kind: TokenKind,
	pub line: u32,
	pub column: u32,
	pub value: &'a [u8],
	pub after_line: bool
}

impl<'a> Token<'a> {
	pub fn new(kind: TokenKind, index: u32, line: u32, column: u32, value: &'a [u8], after_line: bool) -> Token<'a> {
		return Token { index, end_index: index + value.len() as u32, kind, value, line, column, after_line };
	}
	
	pub fn to_string(&self) -> String {
		return String::from_utf8_lossy(self.value).to_string();
	}
}

#[derive(Debug, Clone, Copy)]
enum State {
    Init,
    StringSingleContinue,
    StringDoubleContinue,
    StringSingleEscape,
    StringDoubleEscape,
    Zero,
    HexContinue,
    BinContinue,
    NumContinue,
    NumExpContinue,
    NumFloatContinue,
    Amp,
    Pipe,
    Plus,
    Minus,
    FwdSlash,
    Equal,
    EqualEqual,

    RAngle,
    LAngle,

	Junk,

    JunkSlash,
    JunkNewlineSlash,

    JunkCommentContinue,
    JunkNewlineCommentContinue,

    JunkCommentAsterisk,
    JunkNewlineCommentAsterisk,
}

pub struct TokenStream<'a> {
    buffer: &'a [u8],
    pub index: u32,
	pub line: u32,
	pub column: u32,
    pub len: u32,
	after_line: bool
}

impl<'a> TokenStream<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            index: 0,
			line: 1,
			column: 1,
            len: u32::try_from(buffer.len()).expect("[todo: better error]"),
			after_line: false
        }
    }

	pub fn line(&self, token: Token) -> String {
		let mut ln = String::new();

		let mut index = token.index;

		if index == self.len {
			index -= 1;
		}

		loop {
			if index == 0 {
				break;
			}
			
			if self.buffer[index as usize] == b'\n'  {
				index += 1;
				
				break;
			}

			index -= 1;
		}

		while index < self.len && self.buffer[index as usize] != b'\n' {
			ln += &(self.buffer[index as usize] as char).to_string();

			index += 1;
		}

		return ln;
	}

	pub fn current_line(&self) -> String {
		let mut ln = String::new();

		let mut index = self.index;

		if index == self.len {
			index -= 1;
		}

		while self.buffer[index as usize] != b'\n' && index > 0 {
			index -= 1;
		}

		while index < self.len && self.buffer[index as usize] != b'\n' {
			ln += &(self.buffer[index as usize] as char).to_string();

			index += 1;
		}

		return ln;
	}

	pub fn peek(&mut self) -> Token<'a> {
		let index = self.index;

		let line = self.line;

		let column = self.column;

		let token = self.next();

		self.index = index;

		self.line = line;

		self.column = column;

		return token;
	}

	pub fn get(&mut self) -> u8 {
		if self.index >= self.len {
			return 0;
		}

		self.column += 1;
		
		let c = self.buffer[self.index as usize];

		if c == b'\n' {
			self.column = 1;

			self.line += 1;
		}

		self.index += 1;

		return c;
	}

	pub fn peekc(&mut self) -> u8 {
		if self.index >= self.len {
			return 0;
		}
		
		return self.buffer[self.index as usize];
	}

	pub fn skip_whitespace(&mut self) {
		loop {
			match self.peekc() {
				b' ' | b'\r' | b'\t' => {
					self.get();
				}

				b'\n' => {
					self.after_line = true;

					self.get();
				}

				_ => {
					break;
				}
			}
		}
	}

    pub fn next(&mut self) -> Token<'a> {
		self.skip_whitespace();

		let mut c = self.peekc();

		let mut start = self.index;

		let line = self.line;

		let column = self.column;

		let after_line = self.after_line;

		self.after_line = false;

		if c == 0 {
			return Token::new(TokenKind::Eof, start, line, column, "".as_bytes(), after_line); 
		}

		match c {
			b'+' => {
				self.get();

				match self.peekc() {
					b'+' => {
						self.get();
			
						return Token::new(TokenKind::Increment, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
					}

					_ => {}
				}
			
				return Token::new(TokenKind::Plus, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
			}

			b'-' => {
				self.get();

				match self.peekc() {
					b'-' => {
						self.get();
			
						return Token::new(TokenKind::Decrement, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
					}

					_ => {}
				}
			
				return Token::new(TokenKind::Minus, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
			}

			b'(' => {
				self.get();
			
				return Token::new(TokenKind::LParen, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
			}

			b')' => {
				self.get();
			
				return Token::new(TokenKind::RParen, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
			}

			_ => {}
		}

		if c == b'"' || c == b'\'' {
			self.get();

			start = self.index;

			let mut next: u8 = self.peekc();

			while next != c {
				next = self.get();
				
				if next == 0 {
					// TODO: Better error messages.
					panic!("Unexpected EOF");
				}
			}

			return Token::new(TokenKind::String, start, line, column, &self.buffer[start as usize .. self.index as usize - 1], after_line);
		}

		if c >= b'0' && c <= b'9' {
			self.get();
			
			c = self.peekc();
			
			while c >= b'0' && c <= b'9' {
				c = self.get();
			}

			return Token::new(TokenKind::NumLit, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
		}

		if c.is_ascii_alphabetic() {
			self.get();
			
			while self.peekc().is_ascii_alphanumeric() {
				self.get();
			}
		}

		return Token::new(TokenKind::Ident, start, line, column, &self.buffer[start as usize .. self.index as usize], after_line);
    }
}
