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
}

#[derive(Debug)]
pub struct Hint<'a> {
	pub targets: Vec<Token<'a>>,

	pub column: u32,

	pub help: String,
}

impl<'a> Hint<'a> {
	pub fn new(target: Token<'a>, help: String) -> Hint<'a> {
		return Hint { targets: vec![target], column: target.column, help };
	}
	
	pub fn show(&self, stream: &TokenStream) {
		let mut ptr = String::new();

		for _i in [1..self.column] {
			ptr += &" ";
		}

		ptr += &"^";

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

		for _i in 1..self.column + linecol.len() as u32 {
			ptr += &" ";
		}

		ptr += &"^";

		for obj in &self.targets {
			for _i in obj.index..obj.end_index - 1 {
				ptr += &"~";
			}
		}

		ptr = format!("{}{}{}", color, ptr, Color::Reset);

		println!("{}{}{}{}\n{}\n{}", Color::Yellow, linecol, Color::Reset, stream.line(self.targets[0]), ptr, self.help);
	}
}

#[derive(Debug)]
pub struct SyntaxError<'a> {
	pub message: &'a str,

	pub hints: Vec<Hint<'a>> 
}

impl<'a> SyntaxError<'a> {
	pub fn new(message: &'a str) -> SyntaxError<'a> {
		return SyntaxError { message, hints: vec![] };
	}
}

pub struct Parser<'a> {
	pub stream: TokenStream<'a>
}

impl<'a> Parser<'a> {
	pub fn new(stream: TokenStream<'a>) -> Parser<'a> {
		return Parser { stream: stream };
	}

	pub fn expr(&mut self) -> Result<Expr<'a>, SyntaxError> {
		let expr = self.unary().unwrap();
		
		return self.expr_prec(expr);
	}

	// TODO: Add other operators and precedences.
	pub fn expr_prec(&mut self, lhs: Expr<'a>) -> Result<Expr<'a>, SyntaxError> {
		let mut left = lhs;

		while self.stream.peek().kind == TokenKind::Plus {
			self.stream.next();

			let mut right = self.unary().unwrap();

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

	pub fn unary(&mut self) -> Result<Expr<'a>, SyntaxError> {
		let peek = self.stream.peek();
			
		if peek.kind == TokenKind::Minus {
			self.stream.next();

			return Ok(Expr::expr(ExprKind::UnaryNegate, self.literal().unwrap()));
		} 
		
		return self.prefix();
	}

	pub fn prefix(&mut self) -> Result<Expr<'a>, SyntaxError> {
		let peek = self.stream.peek();

		if peek.kind == TokenKind::Increment || peek.kind == TokenKind::Decrement {
			self.stream.next();

			let kind = if peek.kind == TokenKind::Increment { ExprKind::PreIncrement } else { ExprKind::PreDecrement };

			let target = self.literal().unwrap();

			if self.stream.peek().kind == TokenKind::Increment || self.stream.peek().kind == TokenKind::Decrement {
				// println!("[{}{}{}{}{}] is invalid syntax", Color::Red, peek.to_string(), target.value.unwrap().to_string(), self.stream.peek().to_string(), Color::Reset);

				let mut error = SyntaxError::new("Invalid syntax.");

				error.hints.push(Hint::new(self.stream.peek(), format!("{0}help:{1} remove operator", Color::Blue, Color::Reset)));
				
				return Err(error);
			}

			let expr = Expr::expr(kind, target);

			return Ok(expr);
		}

		return self.postfix();
	}

	pub fn postfix(&mut self) -> Result<Expr<'a>, SyntaxError> {
		let expr = self.literal().unwrap();

		let peek = self.stream.peek();

		if peek.kind == TokenKind::Increment || peek.kind == TokenKind::Decrement {
			self.stream.next();

			let kind = if peek.kind == TokenKind::Increment { ExprKind::PostIncrement } else { ExprKind::PostDecrement };

			return Ok(Expr::expr(kind, expr))
		}

		return Ok(expr);
	}

	pub fn literal(&mut self) -> Result<Expr<'a>, SyntaxError> {
		let next = self.stream.next();

		if next.kind == TokenKind::LParen {
			let expr = self.unary().unwrap();

			if self.stream.next().kind != TokenKind::RParen {
				panic!("Expected )");
			}

			return Ok(Expr::group(expr));
		}
		
		return Ok(Expr::value(ExprKind::Literal, next));
	}
}
