use bril_rs::{Literal, Type, ValueOps};
use std::hash::{Hash, Hasher};

#[derive(PartialEq)]
pub enum Value {
    ValueOp {
        val_type: Type,
        op: ValueOps,
        args: Vec<String>,
    },
    Constant {
        val_type: Type,
        value: Literal,
    },
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::ValueOp { val_type, op, args } => {
                val_type.hash(state);
                op.hash(state);
                args.hash(state);
            }
            Value::Constant { val_type, value } => {
                std::mem::discriminant(value).hash(state);
                val_type.hash(state);
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

impl Eq for Value { }
