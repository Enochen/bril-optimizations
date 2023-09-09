use std::io;

use bbb::{form_blocks, Block, instr_to_txt};
use bril_rs::load_program_from_read;

pub fn print_blocks(blocks: Vec<Block>) {
    for block in &blocks {
        match &block.label {
            Some(label) => println!("[Block: {}]", label),
            None => println!("[Anonymous Block]"),
        }
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
