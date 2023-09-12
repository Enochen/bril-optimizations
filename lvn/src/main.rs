mod table;
mod value;

use bbb::{form_blocks, Block, ToCode};
use bril_rs::{load_program_from_read, output_program, ConstOps, Function, Instruction, ValueOps};
use std::{
    collections::{HashMap, HashSet},
    io,
};
use table::Table;
use util::{CheckOp, SafeAccess};
use value::Value;

fn get_last_writes(block: &Block) -> HashMap<String, usize> {
    block
        .instrs
        .iter()
        .enumerate()
        .filter_map(|(index, instr)| instr.get_dest().map(|dest| (dest, index)))
        .collect()
}

/// Gets variables defined in previous block (and input args)
fn get_outside_vars(block: &mut Block) -> Vec<String> {
    let mut result = HashSet::new();
    let mut defined = HashSet::new();
    for instr in &block.instrs {
        if let Some(args) = instr.get_args() {
            for arg in args {
                if !defined.contains(arg) {
                    result.insert(arg);
                }
            }
        }
        if let Some(dest) = instr.get_dest() {
            defined.insert(dest);
        }
    }
    result.into_iter().cloned().collect()
}

fn apply_lvn_block(block: &mut Block) {
    let mut table = Table::new();
    let mut count = 0;

    for arg in get_outside_vars(block) {
        let value = Value::Unknown { name: arg.clone() };
        table.register_value(&value);
        table.add_binding(&arg, &value);
    }

    let last_writes = get_last_writes(block);
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
            if let (Some(mut dest), Some(mut value)) = (
                instr.get_dest(),
                table.create_value(&instr).map(|v| v.to_canonical()),
            ) {
                if instr.is_call() {
                    table.register_value(&value);
                    table.remove_binding(&dest);
                    table.add_binding(&dest, &value);
                    return instr;
                }
                let simplified = table.simplify(&value);
                if let Value::Constant { kind, literal } = simplified {
                    instr = Instruction::Constant {
                        dest: dest.to_owned(),
                        op: ConstOps::Const,
                        pos: instr.get_pos(),
                        const_type: kind,
                        value: literal,
                    };
                    value = table.create_value(&instr).unwrap();
                }
                // should fold value at this point
                // if not the last write
                //    then add binding for old value
                //    and set instr.dest to lvn_temp_{count}
                let register = table.register_value(&value);
                if index != *last_writes.get(&dest).unwrap() {
                    table.remove_binding(&dest);
                    table.add_binding(&dest, &value);
                    dest = format!("lvn_temp_{}", count);
                    instr.set_dest(dest.to_owned());
                    count += 1;
                }
                if let Some(canonical) = register {
                    if !instr.is_const() {
                        // replace with id to lookup
                        instr = Instruction::Value {
                            dest: dest.to_string(),
                            op_type: instr.get_type().cloned().unwrap(),
                            op: ValueOps::Id,
                            args: vec![canonical],
                            pos: instr.get_pos(),
                            funcs: Vec::new(),
                            labels: Vec::new(),
                        };
                    }
                }
                table.remove_binding(&dest);
                table.add_binding(&dest, &value);
                table.add_candidate(&dest, &value);
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
