use std::collections::{hash_map::Entry, HashMap};

use bril_rs::{Instruction, Literal, Type, ValueOps};
use itertools::Itertools;

use crate::value::Value;
use bril_rs::Literal::*;

#[derive(Debug, Clone)]
struct TableEntry {
    value: Value,
    variables: Vec<String>,
}

pub struct Table {
    entries: Vec<TableEntry>,
    value_index: HashMap<Value, usize>,
    cloud: HashMap<String, usize>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            entries: Vec::new(),
            value_index: HashMap::new(),
            cloud: HashMap::new(),
        }
    }

    fn num_to_value(&self, index: usize) -> &Value {
        &self.entries[index].value
    }

    fn num_to_canonical_var(&self, index: usize) -> Option<&String> {
        self.entries[index].variables.get(0)
    }

    pub fn check_unknown(&self, variable: &str) -> bool {
        self.cloud
            .get(variable)
            .map(|i| self.num_to_value(*i))
            .map(|v| matches!(v, Value::Unknown(_)))
            .unwrap_or(false)
    }

    /// Returns canonical variable name for value referenced by given variable
    pub fn lookup(&self, variable: &str) -> Option<String> {
        // println!("Looking up {}", variable);
        // dbg!(&self.cloud.get(variable));
        // dbg!(self.entries.get(*self.cloud.get(variable).unwrap()));
        self.cloud
            .get(variable)
            .and_then(|index| self.num_to_canonical_var(*index))
            .cloned()
    }

    pub fn register_value(&mut self, value: &Value) {
        if let Entry::Vacant(entry) = self.value_index.entry(value.clone()) {
            entry.insert(self.entries.len());
            self.entries.push(TableEntry {
                value: value.clone(),
                variables: Vec::new(),
            });
        }
    }

    /// Returns canonical variable name if value already exists
    pub fn get_canonical(&mut self, value: &Value) -> Option<String> {
        if let Some(index) = self.value_index.get(value) {
            return self
                .entries
                .get(*index)
                .and_then(|entry| entry.variables.get(0))
                .cloned();
        }
        None
    }

    /// Adds mapping between given variable and value
    pub fn add_binding(&mut self, variable: &str, value: &Value) {
        let index = self.value_index.get(value).unwrap();
        self.cloud.entry(variable.to_owned()).or_insert(*index);
    }

    /// Adds variable as candidate for canonical var for value
    pub fn add_candidate(&mut self, variable: &str, value: &Value) {
        let index = self.value_index.get(value).unwrap();
        self.entries
            .get_mut(*index)
            .unwrap()
            .variables
            .push(variable.to_owned());
    }

    /// Removes mapping for given variable
    pub fn remove_binding(&mut self, variable: &str) {
        if let Some(prev_index) = self.cloud.get(variable) {
            let prev_entry = self.entries.get_mut(*prev_index).unwrap();
            prev_entry.variables.retain(|v| v != variable);
        }
        self.cloud.remove(variable);
    }

    pub fn simplify(&self, value: &Value) -> Value {
        match value {
            Value::Constant { .. } => value.clone(),
            Value::Operation {
                op: ValueOps::Id,
                args,
                ..
            } => {
                let v = &self.entries[args[0]].value;
                return self.simplify(v);
            }
            Value::Operation { op, args, .. } => {
                let simplified_args = args
                    .iter()
                    .map(|num| self.num_to_value(*num))
                    .map(|value| self.simplify(value))
                    .collect_vec();
                if op == &ValueOps::PtrAdd {
                    dbg!(&value);
                    dbg!(&simplified_args);
                }
                match (op, &simplified_args[..]) {
                    // Ints
                    (ValueOps::Add, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Int(a + b))
                    }
                    (ValueOps::Add, [v, Value::Constant(Int(0))]) => v.clone(),
                    (ValueOps::Add, [Value::Constant(Int(0)), v]) => v.clone(),
                    (ValueOps::Sub, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Int(a - b))
                    }
                    (ValueOps::Sub, [v, Value::Constant(Int(0))]) => v.clone(),
                    (ValueOps::Mul, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Int(a * b))
                    }
                    (ValueOps::Mul, [_, Value::Constant(Int(0))]) => Value::Constant(Int(0)),
                    (ValueOps::Mul, [Value::Constant(Int(0)), _]) => Value::Constant(Int(0)),
                    (ValueOps::Mul, [v, Value::Constant(Int(1))]) => v.clone(),
                    (ValueOps::Mul, [Value::Constant(Int(1)), v]) => v.clone(),
                    (ValueOps::Div, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Int(a / b))
                    }
                    (ValueOps::Div, [v, Value::Constant(Int(1))]) => v.clone(),
                    (ValueOps::Div, [a, b]) if a == b => Value::Constant(Int(1)),
                    (ValueOps::Eq, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Bool(a == b))
                    }
                    (ValueOps::Eq, [a, b]) if a == b => Value::Constant(Bool(true)),
                    (ValueOps::Lt, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Bool(a < b))
                    }
                    (ValueOps::Le, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Bool(a <= b))
                    }
                    (ValueOps::Gt, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Bool(a > b))
                    }
                    (ValueOps::Ge, [Value::Constant(Int(a)), Value::Constant(Int(b))]) => {
                        Value::Constant(Bool(a >= b))
                    }
                    // Pointers
                    (ValueOps::PtrAdd, [v, Value::Constant(Int(0))]) => v.clone(),
                    (ValueOps::PtrAdd, [Value::Constant(Int(0)), v]) => v.clone(),
                    // Floats
                    (ValueOps::Fadd, [Value::Constant(Float(a)), Value::Constant(Float(b))]) => {
                        Value::Constant(Float(a + b))
                    }
                    (ValueOps::Fadd, [v, Value::Constant(Float(f))]) if *f == 0.0 => v.clone(),
                    (ValueOps::Fadd, [Value::Constant(Float(f)), v]) if *f == 0.0 => v.clone(),
                    (ValueOps::Fsub, [Value::Constant(Float(a)), Value::Constant(Float(b))]) => {
                        Value::Constant(Float(a - b))
                    }
                    (ValueOps::Fsub, [v, Value::Constant(Float(f))]) if *f == 0.0 => v.clone(),
                    (ValueOps::Fmul, [Value::Constant(Float(a)), Value::Constant(Float(b))]) => {
                        Value::Constant(Float(a * b))
                    }
                    (ValueOps::Fmul, [v, Value::Constant(Float(f))]) if *f == 1.0 => v.clone(),
                    (ValueOps::Fmul, [Value::Constant(Float(f)), v]) if *f == 1.0 => v.clone(),
                    (ValueOps::Fdiv, [Value::Constant(Float(a)), Value::Constant(Float(b))]) => {
                        Value::Constant(Float(a / b))
                    }
                    (ValueOps::Fdiv, [a, b]) if a == b => Value::Constant(Int(1)),
                    // Bools
                    (ValueOps::And, [Value::Constant(Bool(a)), Value::Constant(Bool(b))]) => {
                        Value::Constant(Bool(*a && *b))
                    }
                    (ValueOps::And, [v, Value::Constant(Bool(true))]) => v.clone(),
                    (ValueOps::And, [Value::Constant(Bool(true)), v]) => v.clone(),
                    (ValueOps::Or, [Value::Constant(Bool(a)), Value::Constant(Bool(b))]) => {
                        Value::Constant(Bool(*a || *b))
                    }
                    (ValueOps::Or, [v, Value::Constant(Bool(false))]) => v.clone(),
                    (ValueOps::Or, [Value::Constant(Bool(false)), v]) => v.clone(),
                    (ValueOps::Not, [Value::Constant(Bool(a))]) => Value::Constant(Bool(!a)),
                    _ => value.clone(),
                }
            }
            _ => value.clone(),
        }
    }

    pub fn create_value(&self, instr: &Instruction) -> Option<Value> {
        match instr.clone() {
            Instruction::Constant {
                const_type: Type::Float,
                value: Literal::Int(value),
                ..
            } => Some(Value::Constant(Literal::Float(value as f64))),
            Instruction::Constant { value, .. } => Some(Value::Constant(value)),
            Instruction::Value {
                op: ValueOps::Alloc | ValueOps::Call,
                dest,
                ..
            } => Some(Value::Unknown(dest)),
            Instruction::Value {
                args, op, op_type, ..
            } => Some(Value::Operation {
                kind: op_type,
                op: op,
                args: args
                    .iter()
                    .map(|arg| self.cloud.get(arg).unwrap())
                    .copied()
                    .collect(),
            }),
            Instruction::Effect { .. } => None,
        }
    }
}
