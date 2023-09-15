use bril_rs::{EffectOps, Instruction, Type, ValueOps};

pub trait CheckOp {
    fn is_call(&self) -> bool;
    fn is_const(&self) -> bool;
}

impl CheckOp for Instruction {
    fn is_call(&self) -> bool {
        matches!(
            self,
            Instruction::Effect {
                op: EffectOps::Call,
                ..
            } | Instruction::Value {
                op: ValueOps::Call,
                ..
            }
        )
    }

    fn is_const(&self) -> bool {
        matches!(self, Instruction::Constant { .. })
    }
}

pub trait SafeAccess {
    fn get_args(&self) -> Option<Vec<String>>;
    fn set_args(&mut self, values: Vec<String>);
    fn get_dest(&self) -> Option<String>;
    fn set_dest(&mut self, value: String);
    fn get_type(&self) -> Option<&Type>;
}

impl SafeAccess for Instruction {
    fn get_args(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Value { args, .. } => Some(args.clone()),
            Instruction::Effect { args, .. } => Some(args.clone()),
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

    fn get_dest(&self) -> Option<String> {
        match self {
            Instruction::Constant { dest, .. } => Some(dest.to_string()),
            Instruction::Value { dest, .. } => Some(dest.to_string()),
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

    fn get_type(&self) -> Option<&Type> {
        match self {
            Instruction::Constant { const_type, .. } => Some(const_type),
            Instruction::Value { op_type, .. } => Some(op_type),
            Instruction::Effect { .. } => None,
        }
    }
}
