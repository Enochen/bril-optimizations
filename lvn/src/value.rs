use bril_rs::{Instruction, Literal, Type, ValueOps};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Clone)]
pub enum Value {
    Operation {
        kind: Type,
        op: ValueOps,
        args: Vec<String>,
    },
    Constant {
        kind: Type,
        value: Literal,
    },
}

pub trait ToValue {
    fn to_value(&self) -> Option<Value>;
}

impl ToValue for Instruction {
    fn to_value(&self) -> Option<Value> {
        match self.clone() {
            Instruction::Constant {
                const_type, value, ..
            } => Some(Value::Constant {
                kind: const_type,
                value: value,
            }),
            Instruction::Value {
                args, op, op_type, ..
            } => Some(Value::Operation {
                kind: op_type,
                op: op,
                args: args,
            }),
            Instruction::Effect { .. } => None,
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Operation { kind, op, args } => {
                kind.hash(state);
                op.hash(state);
                args.hash(state);
            }
            Value::Constant { kind, value } => {
                kind.hash(state);
                std::mem::discriminant(value).hash(state);
                match value {
                    Literal::Int(i) => i.hash(state),
                    Literal::Bool(b) => b.hash(state),
                    Literal::Float(f) => f.to_bits().hash(state),
                    Literal::Char(c) => c.hash(state),
                }
            }
        }
    }
}

impl Eq for Value {}
