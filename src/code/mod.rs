use revm::primitives::Bytes;

mod assembler;

pub fn parse(input: &str) -> Bytes {
    // mnemonic or bytecode
    assembler::assemble(input)
    // TODO handle file path or URL
}
