use bbb::{form_blocks, Block, ToCode};
use bril_rs::{load_program_from_read, output_program, Function, Instruction};
use std::{collections::HashSet, io};
use util::{get_args, get_dest};

fn find_used(blocks: &Vec<Block>) -> HashSet<String> {
    blocks
        .iter()
        .flat_map(|block| block.instrs.iter().flat_map(get_args))
        .flatten()
        .cloned()
        .collect()
}

fn trivial_dce_block(block: &mut Block, used: &HashSet<String>) -> bool {
    let original_length = block.instrs.len();
    block.instrs = block
        .instrs
        .iter()
        .filter(|instr| get_dest(instr).map_or(true, |d| used.contains(d)))
        .cloned()
        .collect();
    block.instrs.len() < original_length
}

fn trivial_dce(func: &mut Function) -> bool {
    let mut blocks = form_blocks(func);
    let used = find_used(&blocks);
    let mut dirty = false;
    for block in &mut blocks {
        dirty |= trivial_dce_block(block, &used);
    }
    func.instrs = blocks.to_code();
    dirty
}

fn regular_dce_block(block: &mut Block) -> bool {
    let mut dirty = false;
    let mut dead_vars = HashSet::new();
    let mut new_instrs = Vec::new();
    for instr in block.instrs.iter().rev() {
        if let Some(dest) = get_dest(instr) {
            if dead_vars.contains(dest) {
                dirty = true;
                continue;
            }
            dead_vars.insert(dest);
        }
        if let Some(args) = get_args(instr) {
            args.into_iter().for_each(|arg| {
                dead_vars.remove(&arg);
            });
        }
        new_instrs.push(instr.clone());
    }
    block.instrs = new_instrs.into_iter().rev().collect();
    dirty
}

fn regular_dce(func: &mut Function) -> bool {
    let mut blocks = form_blocks(func);
    let mut dirty = false;
    for block in &mut blocks {
        dirty |= regular_dce_block(block);
    }
    func.instrs = blocks.to_code();
    dirty
}

fn main() -> io::Result<()> {
    let mut program = load_program_from_read(io::stdin());

    // Repeat dce passes until convergence
    while program
        .functions
        .iter_mut()
        .any(|f| trivial_dce(f) || regular_dce(f))
    {}

    output_program(&program);
    Ok(())
}
