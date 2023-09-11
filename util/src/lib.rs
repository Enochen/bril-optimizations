use bril_rs::Instruction;

pub fn get_args(instr: &Instruction) -> Option<&Vec<String>> {
    match instr {
        Instruction::Value { args, .. } => Some(args),
        Instruction::Effect { args, .. } => Some(args),
        _ => None,
    }
}

pub fn set_args(instr: &mut Instruction, values: Vec<String>) {
    match instr {
        Instruction::Value { args, .. } => *args = values,
        Instruction::Effect { args, .. } => *args = values,
        _ => {}
    }
}

pub fn get_dest(instr: &Instruction) -> Option<&String> {
    match instr {
        Instruction::Constant { dest, .. } => Some(dest),
        Instruction::Value { dest, .. } => Some(dest),
        _ => None,
    }
}
