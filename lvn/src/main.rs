mod table;
mod value;

use bbb::{form_blocks, Block, ToCode};
use bril_rs::{load_program_from_read, output_program, EffectOps, Function, Instruction};
use std::io;
use table::Table;
use util::SafeAccess;
use value::ToValue;

fn is_call(instr: &Instruction) -> bool {
    matches!(
        instr,
        Instruction::Effect {
            op: EffectOps::Call,
            ..
        }
    )
}

fn apply_lvn_block(block: &mut Block) {
    let table = Table::new();

    block.instrs = block
        .instrs
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, mut instr)| {
            if let Some(args) = instr.get_args() {
                let canonicalized = args
                    .iter()
                    .map(|arg| table.lookup(arg).unwrap_or_else(|| arg.clone()))
                    .collect();
                instr.set_args(canonicalized);
            }
            if !is_call(&instr) {
                return instr;
            }
            if let (Some(dest), Some(value)) = (instr.get_dest(), instr.to_value()) {
                // should fold value at this point
            }
            instr
        })
        .collect();
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
