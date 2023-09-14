use std::io;

use bbb::{form_blocks, instr_to_txt, Block};
use bril_rs::load_program_from_read;

pub fn print_blocks(blocks: Vec<Block>) {
    for block in &blocks {
        println!("[Block: {}]", block.label);
        for instr in &block.instrs {
            println!("{}", instr_to_txt(instr));
        }
        println!("");
    }
}

fn main() -> io::Result<()> {
    let program = load_program_from_read(io::stdin());
    for func in &program.functions {
        println!("-[Function: {}]-\n", &func.name);
        print_blocks(form_blocks(func));
    }
    Ok(())
}
