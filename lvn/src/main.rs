mod table;
mod value;

use bbb::{form_blocks, Block, ToCode};
use bril_rs::{load_program_from_read, output_program, Function};
use std::io;
use table::Table;
use util::{get_args, set_args, get_dest};

fn apply_lvn_block(block: &mut Block) {
    let table = Table::new();

    let mut new_instrs = Vec::new();
    for (index, instr) in block.instrs.iter_mut().enumerate() {
        if let Some(args) = get_args(instr) {
            let canonicalized = args
                .iter()
                .map(|arg| table.lookup(arg).unwrap_or_else(|| arg.clone()))
                .collect();
            set_args(instr, canonicalized)
        }
        if let Some(dest) = get_dest(instr) {

        }
    }
    block.instrs = new_instrs;
}

fn apply_lvn(func: &mut Function) {
    let mut blocks = form_blocks(func);
    for block in &mut blocks {
        apply_lvn_block(block);
    }
    func.instrs = blocks.to_code();
}

fn main() -> io::Result<()> {
    let mut program = load_program_from_read(io::stdin());

    program.functions.iter_mut().for_each(apply_lvn);

    output_program(&program);
    Ok(())
}
