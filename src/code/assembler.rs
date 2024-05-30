use revm::{
    interpreter::{
        opcode::{OpCodeInfo, PUSH0},
        OPCODE_INFO_JUMPTABLE,
    },
    primitives::{hex, Bytes, U256},
};
use std::{collections::HashMap, str::FromStr};

#[allow(dead_code)]
pub fn assemble(input: &str) -> Bytes {
    let mut opcodes = HashMap::new();

    for (opcode, info) in OPCODE_INFO_JUMPTABLE.into_iter().enumerate() {
        if let Some(info) = info {
            opcodes.insert(info.name().to_owned(), (opcode as u8, info));
            opcodes.insert(info.name().to_ascii_lowercase(), (opcode as u8, info));
        }
    }

    let mut lexer = Lexer::new(input, opcodes);
    let mut bytecode = vec![];

    while let Some(token) = lexer.next_token() {
        let mut bytes = lexer.parse_token(token);
        bytecode.append(&mut bytes);
    }

    Bytes::from(bytecode)
}

pub struct Lexer<'a> {
    input: &'a str,
    current_position: usize,
    opcodes: HashMap<String, (u8, OpCodeInfo)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, opcodes: HashMap<String, (u8, OpCodeInfo)>) -> Self {
        Self {
            input,
            current_position: 0,
            opcodes,
        }
    }

    fn next_token(&mut self) -> Option<&'a str> {
        let mut start = self.current_position;

        let mut comment_line = false;
        loop {
            let char = self.input.chars().nth(self.current_position);

            // do not process commented source code line
            if comment_line {
                if char != Some('\n') {
                    self.current_position += 1;
                    continue;
                } else {
                    comment_line = false;
                }
            }

            match char {
                Some('#') => {
                    comment_line = true;
                    self.current_position += 1;
                    continue;
                }
                Some(' ' | '\t' | '\n') => {
                    if start == self.current_position {
                        // trim leading whitespaces
                        self.current_position += 1;
                        start += 1;
                        continue;
                    } else {
                        // collect token just before next whitespace
                        let token = &self.input[start..self.current_position];
                        return Some(token);
                    }
                }
                Some(_) => {
                    self.current_position += 1;
                    continue;
                }
                None => {
                    if start != self.current_position {
                        return Some(&self.input[start..self.current_position]);
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    fn parse_token(&self, token: &str) -> Vec<u8> {
        // opcode?
        if let Some((opcode, _)) = self.opcodes.get(token) {
            return vec![*opcode];
        }
        // hex?
        if token.starts_with("0x") {
            let mut bytes = hex::decode(token).unwrap();
            if bytes.len() > 32 {
                println!("Hex length exceeds 32 bytes -> {}", token);
                std::process::exit(1);
            }
            prepend_push_opcode(&mut bytes);
            return bytes;
        }
        // decimal?
        if let Some(stripped) = token.strip_prefix("0t") {
            if let Ok(bn) = U256::from_str(stripped) {
                let mut bytes = bn.to_be_bytes_vec();
                if bytes.len() > 32 {
                    println!("Decimal value bigger than 256 bits -> {}", token);
                    std::process::exit(1);
                }
                prepend_push_opcode(&mut bytes);
                return bytes;
            } else {
                println!("Invalid decimal value -> {}", token);
                std::process::exit(1);
            }
        }
        // raw bytes?
        if let Ok(bytes) = hex::decode(token) {
            return bytes;
        }
        panic!("Unexpected token -> {}", token);
    }
}

fn prepend_push_opcode(bytes: &mut Vec<u8>) {
    while !bytes.is_empty() && bytes[0] == 0 {
        bytes.remove(0);
    }
    if bytes.is_empty() {
        bytes.push(PUSH0);
    } else {
        let push_opcode = 0x60 + (bytes.len() as u8 - 1);
        bytes.insert(0, push_opcode);
    }
}

mod test {
    #[test]
    fn test_lexer_1_push_hex_and_decimals() {
        let input = "0x01 0t1023 MSTORE";
        let bytecode = super::assemble(input);
        assert_eq!(bytecode, revm::primitives::bytes!("60016103ff52"));
    }

    #[test]
    fn test_lexer_2_place_raw_bytes() {
        let input = "PUSH1 01 0x2002 MSTORE";
        let bytecode = super::assemble(input);
        assert_eq!(bytecode, revm::primitives::bytes!("600161200252"));
    }
}
