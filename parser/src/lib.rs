use tokenizer::{Token, TokenKind, TokenStream};

use std::vec::Vec;

#[derive(Debug)]
pub enum ExprKind {
	Literal,
	UnaryNegate,
	Group,
	Add,
	PreIncrement,
	PostIncrement
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

pub struct Parser<'a> {
	pub stream: TokenStream<'a>
}

impl<'a> Parser<'a> {
	pub fn new(stream: TokenStream<'a>) -> Parser<'a> {
		return Parser { stream: stream };
	}

	pub fn add(&mut self) -> Expr<'a> {
		let left = self.unary();

		if self.stream.peek().kind == TokenKind::Plus {
			self.stream.next();

			let mut expr = Expr::new(ExprKind::Add);

			expr.data.push(left);

			expr.data.push(self.unary());

			return expr;
		}

		return left;
	}

	pub fn unary(&mut self) -> Expr<'a> {
		let peek = self.stream.peek();
			
		if peek.kind == TokenKind::Minus {
			self.stream.next();

			return Expr::expr(ExprKind::UnaryNegate, self.literal());
		} 

		if peek.kind == TokenKind::Increment {
			self.stream.next();

			return Expr::expr(ExprKind::PreIncrement, self.literal());
		} 

		let expr = self.literal();

		if self.stream.peek().kind == TokenKind::Increment {
			self.stream.next();

			return Expr::expr(ExprKind::PostIncrement, expr);
		}

		return expr;
	}

	pub fn literal(&mut self) -> Expr<'a> {
		let next = self.stream.next();

		if next.kind == TokenKind::LParen {
			let expr = self.unary();

			if self.stream.next().kind != TokenKind::RParen {
				panic!("Expected )");
			}

			return Expr::group(expr);
		}
		
		return Expr::value(ExprKind::Literal, next);
	}
}
