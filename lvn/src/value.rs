use bril_rs::{Literal, Type, ValueOps};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Operation {
        kind: Type,
        op: ValueOps,
        args: Vec<usize>,
    },
    Constant(Literal),
    Unknown(String),
}

impl Value {
    pub fn to_canonical(&self) -> Value {
        let mut canonical = self.clone();
        if let Value::Operation {
            args,
            op: ValueOps::Add | ValueOps::Mul,
            ..
        } = &mut canonical
        {
            args.sort();
        }
        canonical
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
            Value::Constant(literal) => {
                std::mem::discriminant(literal).hash(state);
                match literal {
                    Literal::Int(i) => i.hash(state),
                    Literal::Bool(b) => b.hash(state),
                    Literal::Float(f) => f.to_bits().hash(state),
                    Literal::Char(c) => c.hash(state),
                }
            }
            Value::Unknown(name) => name.hash(state),
        }
    }
}

impl Eq for Value {}
