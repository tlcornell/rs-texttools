use std::io;
use std::io::prelude::*;

static IN_TOKEN: u8 = 1;
static BTWN_TOKS: u8 = 0;
static END_OF_STRING: char = '\0';


#[derive(Debug)]
struct CharData {
    ch: char,
    byte_offset: usize,
    char_offset: usize,
}

/////////////////////////////////////////////////////////
// Probably CharDataIter could be replaced by a clever
// call to map() on the underlying char iterator...
//
struct CharDataIter<'a> {
    char_stream: &'a mut Iterator<Item = char>,
    byte_offset: usize,
    char_offset: usize,
    really_done: bool,
}

impl<'a> CharDataIter<'a> {
    fn new(chs: &'a mut Iterator<Item = char>) -> Self {
        CharDataIter {
            char_stream: chs,
            byte_offset: 0,
            char_offset: 0,
            really_done: false,
        }
    }
}

impl<'a> Iterator for CharDataIter<'a> {
    type Item = CharData;

    fn next(&mut self) -> Option<Self::Item> {
        match self.char_stream.next() {
            Some(c) => {
                let result = CharData {
                    ch: c,
                    byte_offset: self.byte_offset,
                    char_offset: self.char_offset,
                };
                self.char_offset += 1;
                self.byte_offset += c.len_utf8();
                Some(result)
            },
            None => {
                if self.really_done {
                    None
                } else {
                    // Special <end-of-string> marker
                    self.really_done = true;
                    Some (
                        CharData {
                            ch: END_OF_STRING,  // should be ignored!
                            byte_offset: self.byte_offset,
                            char_offset: self.char_offset,
                        }
                    )
                }
            }
        }
    }
}
//
// CharDataIter
/////////////////////////////////////////////////////////



/////////////////////////////////////////////////////////
// TokenIter
//

#[derive(Debug)]
struct Token {
    text: String,
    byte_offsets: (usize, usize),
    char_offsets: (usize, usize),
    token_offset: usize
}

impl Token {
    fn new() -> Token {
        Token {
            text: "".to_string(),
            byte_offsets: (0, 0),
            char_offsets: (0, 0),
            token_offset: 0,
        }
    }
}

struct TokenIter<'a> {
    chdat_stream: &'a mut CharDataIter<'a>,
    curr_tok_offset: usize,
    state: u8,
}

impl<'a> TokenIter<'a> {
    fn new(chdats: &'a mut CharDataIter<'a>) -> Self {
        TokenIter {
            chdat_stream: chdats,
            curr_tok_offset: 0,
            state: BTWN_TOKS,
        }
    }

    fn is_boundary_char(ch: char) -> bool {
        if ch == END_OF_STRING {
            true
        } else if ch.is_whitespace() {
            true
        } else {
            false
        }
    }
}

/*  Always start out BTWN_TOKS, and therefore always end in BTWN_TOKS.
    Start by skipping characters until state changes to IN_TOKEN.
    Then (1) set the token start offsets; (2) march the char data iter forward
    until state changes to BTWN_TOKS, then fix the end offsets of the token
    under construction. Update the current token offset.
    Leave the resulting Token as the return value of next().

    If the underlying CharDataIter yields END_OF_SENTENCE:
        IN_TOKEN --> ship the current token
        BTWN_TOKS --> return None
    In the first case, the next call to next() will immediately trigger
    the second case.
*/

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        assert_eq!(self.state, BTWN_TOKS);
        let mut curr_tok = Token::new();
        loop {
            match self.chdat_stream.next() {

                Some( CharData {ch, byte_offset, char_offset} ) => {

                    if TokenIter::is_boundary_char(ch) {
                        if self.state == IN_TOKEN {
                            // ship token
                            curr_tok.byte_offsets.1 = byte_offset;
                            curr_tok.char_offsets.1 = char_offset;
                            self.state = BTWN_TOKS;
                            self.curr_tok_offset += 1;
                            return Some(curr_tok);
                        }
                        // else do nothing -- skip boundary chars
                    } else {
                        if self.state == BTWN_TOKS {
                            // start token 
                            curr_tok.token_offset = self.curr_tok_offset;
                            curr_tok.byte_offsets.0 = byte_offset;
                            curr_tok.char_offsets.0 = char_offset;
                            self.state = IN_TOKEN;
                        }
                        // Accumulate characters
                        curr_tok.text.push(ch);
                        curr_tok.byte_offsets.1 = byte_offset;
                        curr_tok.char_offsets.1 = char_offset;
                    }
                },

                None => {
                    // May need to ship a token here!
                    if self.state == IN_TOKEN {
                        self.state = BTWN_TOKS;
                        return Some(curr_tok);
                    }
                    return None;
                }
            }
        }
    }
}
//
// TokenIter
/////////////////////////////////////////////////////////


fn main() {
    // Get stdin into a string
    let stdin = io::stdin();
    let mut s = String::new();
    stdin.lock().read_to_string(&mut s).unwrap();
    println!("{}", s);

    // Construct a tokenizer by adapting some more primitive iterators
    let mut chs = s.chars();
    let mut chds = CharDataIter::new(&mut chs);
    let mut toks = TokenIter::new(&mut chds);

    // Run the tokenizer, dump debug info for each token:
    loop {
        match toks.next() {
            Some(tok) => { println!("{:?}", tok) },
            None => { println!("END_OF_TEXT"); break; }
        }
    }
}

