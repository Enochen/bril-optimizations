use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use bril_rs::{Literal, Type, ValueOps};

use crate::value::Value;

struct TableEntry {
    value: Value,
    variables: Vec<String>,
}

pub struct Table {
    value_index: HashMap<Value, Rc<RefCell<TableEntry>>>,
    cloud: HashMap<String, Rc<RefCell<TableEntry>>>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            value_index: HashMap::new(),
            cloud: HashMap::new(),
        }
    }

    /// Returns canonical variable name for value referenced by given variable
    pub fn lookup(&self, variable: &str) -> Option<String> {
        self.cloud
            .get(variable)
            .and_then(|rc| rc.borrow().variables.first().cloned())
    }

    /// Returns true if value was newly inserted
    pub fn register_value(&mut self, value: Value) -> bool {
        match self.value_index.entry(value.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(Rc::new(RefCell::new(TableEntry {
                    value,
                    variables: vec![],
                })));
                true
            }
        }
    }

    /// Removes mapping for given variable
    pub fn remove_binding(&mut self, variable: &str) {
        if let Some(prev_entry) = self.cloud.get(variable) {
            prev_entry.borrow_mut().variables.retain(|v| v != variable);
            if prev_entry.borrow().variables.is_empty() {
                self.value_index.remove(&prev_entry.borrow().value);
            }
        }
        self.cloud.remove(variable);
    }

    /// Adds mapping between given variable and value
    /// Precondition: Value maps to an entry
    pub fn add_binding(&mut self, variable: &str, value: &Value) {
        if let Some(entry) = self.value_index.get(value) {
            self.cloud.insert(variable.to_owned(), Rc::clone(entry));
        }
    }
}
