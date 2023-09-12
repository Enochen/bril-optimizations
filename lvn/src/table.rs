use std::collections::{hash_map::Entry, HashMap};

use bril_rs::{Instruction, Literal, Type, ValueOps};
use itertools::Itertools;

use crate::value::Value;

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

    /// Returns canonical variable name if value already exists
    pub fn register_value(&mut self, value: &Value) -> Option<String> {
        match self.value_index.entry(value.clone()) {
            Entry::Occupied(entry) => self
                .entries
                .get(*entry.get())
                .and_then(|entry| entry.variables.get(0))
                .cloned(),
            Entry::Vacant(entry) => {
                entry.insert(self.entries.len());
                let mut variables = Vec::new();
                // if let Value::Unknown { name } = value.clone() {
                //     variables.push(name.to_owned());
                // }
                self.entries.push(TableEntry {
                    value: value.clone(),
                    variables,
                });
                None
            }
        }
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
            Value::Operation {
                op: ValueOps::Add,
                args,
                ..
            } => {
                let (a, b) = args
                    .iter()
                    .map(|num| self.num_to_value(*num))
                    .map(|value| self.simplify(value))
                    .collect_tuple()
                    .expect("there are two args");
                match (a, b) {
                    (
                        Value::Constant {
                            kind: Type::Int,
                            literal: Literal::Int(a),
                        },
                        Value::Constant {
                            kind: Type::Int,
                            literal: Literal::Int(b),
                        },
                    ) => Value::Constant {
                        kind: Type::Int,
                        literal: Literal::Int(a + b),
                    },
                    _ => value.clone(),
                }
            }
            _ => value.clone(),
        }
    }

    pub fn create_value(&self, instr: &Instruction) -> Option<Value> {
        match instr.clone() {
            Instruction::Constant {
                const_type, value, ..
            } => Some(Value::Constant {
                kind: const_type,
                literal: value,
            }),
            Instruction::Value {
                op: ValueOps::Alloc | ValueOps::Call,
                dest,
                ..
            } => Some(Value::Unknown { name: dest }),
            Instruction::Value {
                args, op, op_type, ..
            } => Some(Value::Operation {
                kind: op_type,
                op: op,
                args: args
                    .iter()
                    .flat_map(|arg| self.cloud.get(arg))
                    .copied()
                    .collect(),
            }),
            Instruction::Effect { .. } => None,
        }
    }
}
