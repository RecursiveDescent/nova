use tokenizer::{TokenKind, TokenStream};

pub fn main() {
    let input = "2 == asdf 23 a ";
    let mut stream = TokenStream::new(input.as_bytes());

    loop {
        let token = stream.next();

		if (token.kind == TokenKind::Eof) {
			break;
		}
		
        println!("{:?} {:?}", token.kind, &input[token.start as usize .. stream.index as usize]);
		
        if token.kind == TokenKind::Eof {
            break;
        }
    }
}
