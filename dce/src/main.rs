use bbb::{blocks_to_code, form_blocks, Block};
use bril_rs::{load_program_from_read, output_program, Function, Instruction, Program};
use std::{collections::HashSet, io};

fn get_args(instr: &Instruction) -> Option<Vec<String>> {
    match instr {
        Instruction::Value { args, .. } => Some(args.clone()),
        Instruction::Effect { args, .. } => Some(args.clone()),
        _ => None,
    }
}

fn get_dest(instr: &Instruction) -> Option<&String> {
    match instr {
        Instruction::Constant { dest, .. } => Some(dest),
        Instruction::Value { dest, .. } => Some(dest),
        _ => None,
    }
}

fn find_used(blocks: &Vec<Block>) -> HashSet<String> {
    blocks
        .iter()
        .flat_map(|block| block.instrs.iter().flat_map(get_args))
        .flatten()
        .collect()
}

fn trivial_dce(func: &mut Function) -> bool {
    let mut blocks = form_blocks(func);
    let used = find_used(&blocks);
    let mut dirty = false;
    for block in &mut blocks {
        let original_length = block.instrs.len();
        block
            .instrs
            .retain(|instr| get_dest(instr).map_or(true, |dest| used.contains(dest)));
        dirty |= block.instrs.len() < original_length;
    }
    func.instrs = blocks_to_code(&blocks.clone());
    dirty
}

fn global_dce(program: &Program) {}

fn main() -> io::Result<()> {
    let mut program = load_program_from_read(io::stdin());

    for func in &mut program.functions {
        trivial_dce(func);
    }

    global_dce(&program);

    output_program(&program);
    Ok(())
}
