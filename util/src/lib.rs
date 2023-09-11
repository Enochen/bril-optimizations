use bril_rs::Instruction;

pub trait SafeAccess {
    fn get_args(&self) -> Option<&Vec<String>>;
    fn set_args(&mut self, values: Vec<String>);
    fn get_dest(&self) -> Option<&String>;
    fn set_dest(&mut self, value: String);
}

impl SafeAccess for Instruction {
    fn get_args(&self) -> Option<&Vec<String>> {
        match self {
            Instruction::Value { args, .. } => Some(args),
            Instruction::Effect { args, .. } => Some(args),
            Instruction::Constant { .. } => None,
        }
    }

    fn set_args(&mut self, values: Vec<String>) {
        match self {
            Instruction::Value { args, .. } => *args = values,
            Instruction::Effect { args, .. } => *args = values,
            Instruction::Constant { .. } => {}
        }
    }

    fn get_dest(&self) -> Option<&String> {
        match self {
            Instruction::Constant { dest, .. } => Some(dest),
            Instruction::Value { dest, .. } => Some(dest),
            Instruction::Effect { .. } => None,
        }
    }

    fn set_dest(&mut self, value: String) {
        match self {
            Instruction::Constant { dest, .. } => *dest = value,
            Instruction::Value { dest, .. } => *dest = value,
            Instruction::Effect { .. } => {}
        }
    }
}
