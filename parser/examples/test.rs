use parser::{ Parser, Expr, ExprKind, Color };
use tokenizer::TokenStream;

pub fn visualize(expr: &Expr, indent: u32) -> String {
	let mut tabs: String = "".to_string();

	for _i in [0..indent] {
		tabs += &"\t".to_string();
	}

	let mut value: String = String::new();

	match expr.kind {
		ExprKind::Add => {
			value += &"(".to_string();
		
			value += &visualize(expr.data.get(0).unwrap(), indent);
	
			value += &" + ".to_string();
	
			value += &visualize(expr.data.get(1).unwrap(), indent);
	
			value += &")".to_string();
	
			return value;
		}

		 ExprKind::PreIncrement => {
			 value += &"++".to_string();
		
			value += &visualize(expr.expr.as_ref().unwrap().as_ref(), indent);
	
			return value;
		 }

		ExprKind::PostIncrement => {
			value += &visualize(expr.expr.as_ref().unwrap().as_ref(), indent);

			value += &"++".to_string();
	
			return value;
		}

		ExprKind::PreDecrement => {
			 value += &"--".to_string();
		
			value += &visualize(expr.expr.as_ref().unwrap().as_ref(), indent);
	
			return value;
		 }

		ExprKind::PostDecrement => {
			value += &visualize(expr.expr.as_ref().unwrap().as_ref(), indent);

			value += &"--".to_string();
	
			return value;
		}

		_ => {}
	}

	return expr.value.unwrap().to_string();
}

pub fn main() {
    let input = "++err-- + 2";

	let mut stream = TokenStream::new(input.as_bytes());
	
	let mut parser = Parser::new(stream);

	let add = parser.expr();

	if add.is_err() {
		add.as_ref().unwrap_err().hints[0].underline_tokens(&stream, Color::Red);

		return;
	}

	println!("{}", visualize(&add.unwrap(), 0));

	// println!("{:?}\n", parser.add());

	// let expr = add.data.get(1);

	// println!("{:?}\n", expr.unwrap().expr.as_ref().unwrap().value.unwrap().to_string());
}
