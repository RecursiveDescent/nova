use parser::Parser;
use tokenizer::TokenStream;

pub fn main() {
    let input = "2 + \n ++2";

	let stream = TokenStream::new(input.as_bytes());

	let mut parser = Parser::new(stream);

	let add = parser.add();

	println!("{:?}\n", add);

	let expr = add.data.get(1);

	println!("{:?}\n", (*expr.unwrap().expr.unwrap()).value);
}
