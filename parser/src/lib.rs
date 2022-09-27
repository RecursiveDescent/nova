use tokenizer::{Token, TokenKind, TokenStream};

use std::vec::Vec;

#[derive(Debug, Eq, PartialEq)]
pub enum ExprKind {
	Literal,
	UnaryNegate,
	Group,
	Add,
	PreIncrement,
	PostIncrement,
	PreDecrement,
	PostDecrement,
}

pub mod Color {
	pub const Red: &str = "\x1b[31m";

	pub const Green: &str = "\x1b[32m";

	pub const Yellow: &str = "\x1b[33m";

	pub const Blue: &str = "\x1b[34m";

	pub const Reset: &str = "\x1b[0m";
}

#[derive(Debug)]
pub struct Expr<'a> {
	pub kind: ExprKind,
	
	pub data: Vec<Expr<'a>>,

	pub expr: Option<Box<Expr<'a>>>,

	pub value: Option<Token<'a>>
}

impl<'a> Expr<'a> {
	pub fn new(kind: ExprKind) -> Expr<'a> {
		return Expr {
			kind,
			
			data: Vec::<Expr>::new(),

			expr: None,

			value: None
		};
	}

	pub fn expr(kind: ExprKind, expr: Expr<'a>) -> Expr<'a> {
		return Expr {
			kind,
			
			data: Vec::<Expr>::new(),

			expr: Some(Box::<Expr<'a>>::new(expr)),

			value: None
		};
	}

	pub fn value(kind: ExprKind, value: Token<'a>) -> Expr<'a> {
		return Expr {
			kind,
			
			data: Vec::<Expr>::new(),

			expr: None,

			value: Some(value)
		};
	}

	pub fn group(expr: Expr<'a>) -> Expr<'a> {
		return Expr {
			kind: ExprKind::Group,
			
			data: Vec::<Expr>::new(),

			expr: Some(Box::<Expr<'a>>::new(expr)),

			value: None
		};
	}

	pub fn last_token(expr: &Expr<'a>) -> Token<'a> {
		match expr.kind {
			ExprKind::Add => {
				return Expr::last_token(&expr.data[1]);
			}

			ExprKind::PreDecrement | ExprKind::PostDecrement | ExprKind::PreIncrement | ExprKind::PostIncrement => {
				return Expr::last_token(expr.expr.as_ref().unwrap());
			}

			ExprKind::Literal => {
				return expr.value.unwrap();
			}

			ExprKind::Group => {
				return Expr::last_token(expr.expr.as_ref().unwrap());
			}

			_ => { panic!("{:?}", expr.kind); }
		}
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HintKind {
	Add,
	Remove
}

#[derive(Debug, Clone)]
pub struct Hint<'a> {
	pub kind: HintKind,
	
	pub targets: Vec<Token<'a>>,

	pub column: u32,

	pub help: String,
}

impl<'a> Hint<'a> {
	pub fn new(kind: HintKind, targets: Vec<Token<'a>>, help: String) -> Hint<'a> {
		if targets.len() == 0 {
			panic!("targets");
		}

		let first = targets[0];
		
		return Hint { kind, targets, column: first.column, help };
	}

	pub fn end(&self) -> u32 {
		if self.targets.len() == 1 {
			return self.targets[0].column + self.targets[0].value.len() as u32;
		}

		return 0;
	}
	
	pub fn show(&self, stream: &TokenStream) {
		if self.kind == HintKind::Remove {
			self.underline_tokens(stream, Color::Red);

			return;
		}
		
		let mut ptr = String::new();

		for _i in 1..self.column {
			ptr += &" ";
		}

		ptr += &"^";

		if self.kind == HintKind::Add {
			ptr = format!("{}{}{}", Color::Green, ptr, Color::Reset);
		}

		println!("{}\n{}\n{}", stream.line(self.targets[0]), ptr, self.help);
	}

	pub fn underline(&self, stream: &TokenStream, from: u32, to: u32, color: &str) {
		let mut ptr = String::new();

		for _i in 1..from {
			ptr += &" ";
		}

		ptr += &"^";

		for _i in from..to {
			ptr += &"~";
		}

		println!("{}\n{}{}{}\n{}", stream.line(self.targets[0]), color, ptr, Color::Reset, self.help);
	}

	pub fn underline_token(&self, stream: &TokenStream, token: Token) {
		let mut ptr = String::new();

		for _i in token.index..token.end_index - 1 {
			ptr += &"~";
		}

		println!("{}\n{}\n{}", stream.line(self.targets[0]), ptr, self.help);
	}

	pub fn underline_tokens(&self, stream: &TokenStream, color: &str) {
		let mut ptr = String::new();

		let linecol = format!("{}:{}: ", self.targets[0].line, self.targets[0].column);

		for _i in 1..self.column {
			ptr += &" ";
		}

		ptr += &"^";

		for obj in &self.targets {
			for _i in obj.index..obj.end_index - 1 {
				ptr += &"~";
			}
		}

		ptr = format!("{}{}{}", color, ptr, Color::Reset);

		println!("{}\n{}\n{}", stream.line(self.targets[0]), ptr, self.help);
	}
}

#[derive(Debug)]
pub struct SyntaxError<'a> {
	pub subject: Token<'a>,
	
	pub message: &'a str,

	pub hints: Vec<Hint<'a>>
}

impl<'a> SyntaxError<'a> {
	pub fn new(stream: &TokenStream, subject: Token<'a>, message: &'a str) -> SyntaxError<'a> {
		let mut error = SyntaxError { subject, message, hints: vec![] };

		return error;
	}

	pub fn underline(&mut self, stream: &TokenStream, from: u32, to: u32, color: &str) {
		let mut ptr = String::new();

		for _i in 1..from {
			ptr += &" ";
		}

		ptr += &"^";

		for _i in from..to {
			ptr += &"~";
		}

		println!("{}{}{}", color, ptr, Color::Reset);
	}

	pub fn show(&mut self, stream: &TokenStream<'a>) {
		println!("{}{}:{}{}: {}\n", Color::Yellow, self.subject.line, self.subject.column, Color::Reset, self.message);

		// self.hints[0].show(stream);

		let mut help: Vec<String> = Vec::new();

		println!("{}", stream.line(self.subject));

		for hint in self.hints.clone() {
			help.push(hint.help.clone());
			
			match hint.kind {
				HintKind::Add => {
					self.underline(stream, hint.column, hint.end() - 1, Color::Green);
				}

				HintKind::Remove => {
					self.underline(stream, hint.column, hint.end() - 1, Color::Red);
				}

				_ => {}
			}
		}

		for help in help {
			println!("{}", help);
		}
	}
}

pub struct Parser<'a> {
	pub stream: TokenStream<'a>
}

impl<'a> Parser<'a> {
	pub fn new(stream: TokenStream<'a>) -> Parser<'a> {
		return Parser { stream: stream };
	}

	pub fn expr(&mut self) -> Result<Expr<'a>, SyntaxError<'a>> {
		let expr = self.unary();

		if expr.is_err() {
			return expr;
		}
		
		return self.expr_prec(expr.unwrap());
	}

	// TODO: Add other operators and precedences.
	pub fn expr_prec(&mut self, lhs: Expr<'a>) -> Result<Expr<'a>, SyntaxError<'a>> {
		let mut left = lhs;

		while self.stream.peek().kind == TokenKind::Plus {
			self.stream.next();

			let mut right = match self.unary() {
				Ok(expr) => {
					expr
				}
				
				Err(error) => {
					return Err(error); 
				}
			};

			if self.stream.peek().kind == TokenKind::Plus {
				self.stream.next();
				
				let mut expr = Expr::new(ExprKind::Add);

				expr.data.push(left);
	
				expr.data.push(right);
	
				left = expr;

				right = self.unary().unwrap();
			}

			while self.stream.peek().kind == TokenKind::Plus {
				right = self.expr_prec(right).unwrap();
			} 

			let mut expr = Expr::new(ExprKind::Add);

			expr.data.push(left);

			expr.data.push(right);

			left = expr;
		}

		return Ok(left);
	}

	pub fn unary(&mut self) -> Result<Expr<'a>, SyntaxError<'a>> {
		let peek = self.stream.peek();
			
		if peek.kind == TokenKind::Minus {
			self.stream.next();

			return Ok(Expr::expr(ExprKind::UnaryNegate, self.literal().unwrap()));
		} 
		
		return self.prefix();
	}

	pub fn prefix(&mut self) -> Result<Expr<'a>, SyntaxError<'a>> {
		let peek = self.stream.peek();

		if peek.kind == TokenKind::Increment || peek.kind == TokenKind::Decrement {
			self.stream.next();

			let kind = if peek.kind == TokenKind::Increment { ExprKind::PreIncrement } else { ExprKind::PreDecrement };

			let target = self.literal().unwrap();

			if self.stream.peek().kind == TokenKind::Increment || self.stream.peek().kind == TokenKind::Decrement {
				// println!("[{}{}{}{}{}] is invalid syntax", Color::Red, peek.to_string(), target.value.unwrap().to_string(), self.stream.peek().to_string(), Color::Reset);

				let next = self.stream.peek();

				let mut error = SyntaxError::new(&self.stream, next, "Invalid syntax.");

				error.hints.push(Hint::new(HintKind::Remove, vec![self.stream.peek()], format!("{0}help:{1} remove operator", Color::Blue, Color::Reset)));

				self.stream.next();
				
				return Err(error);
			}

			let expr = Expr::expr(kind, target);

			return Ok(expr);
		}

		return self.postfix();
	}

	pub fn postfix(&mut self) -> Result<Expr<'a>, SyntaxError<'a>> {
		let expr = match self.literal() {
			Ok(expr) => {
				expr
			}
			
			Err(error) => {
				return Err(error); 
			}
		};

		let peek = self.stream.peek();

		if peek.kind == TokenKind::Increment || peek.kind == TokenKind::Decrement {
			self.stream.next();

			let kind = if peek.kind == TokenKind::Increment { ExprKind::PostIncrement } else { ExprKind::PostDecrement };

			return Ok(Expr::expr(kind, expr))
		}

		return Ok(expr);
	}

	pub fn literal(&mut self) -> Result<Expr<'a>, SyntaxError<'a>> {
		let next = self.stream.next();

		if next.kind == TokenKind::LParen {
			let expr = match self.expr() {
				Ok(expr) => {
					expr
				}
				
				Err(error) => {
					return Err(error); 
				}
			};

			if self.stream.next().kind != TokenKind::RParen {
				let last = Expr::last_token(&expr);
				
				let mut error = SyntaxError::new(&self.stream, last, "Unclosed parenthesis after expression.");

				error.hints.push(Hint::new(HintKind::Add, vec![last], format!("{0}help:{1} add ) after expression", Color::Blue, Color::Reset)));

				return Err(error);
			}

			return Ok(Expr::group(expr));
		}
		
		return Ok(Expr::value(ExprKind::Literal, next));
	}
}
