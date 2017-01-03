use std::io;
use std::io::prelude::*;

mod chardata;
mod toksiter;



fn main() {
    // Get stdin into a string
    let stdin = io::stdin();
    let mut s = String::new();
    stdin.lock().read_to_string(&mut s).unwrap();
    println!("{}", s);

    // Construct a tokenizer by adapting some more primitive iterators
    let mut chs = s.chars();
    let mut chds = chardata::CharDataIter::new(&mut chs);
    let mut toks = toksiter::TokenIter::new(&mut chds);

    // Run the tokenizer, dump debug info for each token:
    loop {
        match toks.next() {
            Some(tok) => { println!("{:?}", tok) },
            None => { println!("<END_OF_TEXT>"); break; }
        }
    }
}

