use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use bril_rs::{Instruction, ValueOps};
use cfg::{CFGNode, DataFlowHelpers, CFG};
use dom::{DomResult, DominatorUtil};
use itertools::Itertools;
use util::SafeAccess;

fn insert_phi_nodes(cfg: &mut CFG, dom_result: &DomResult<CFGNode>) {
    let mut defs = cfg.get_defs();
    let mut seen: HashMap<&String, HashSet<cfg::CFGNode>> = HashMap::new();
    for (variable, def_nodes) in defs.clone().iter().sorted_by_key(|x| Reverse(x.0)) {
        for def_node in def_nodes {
            let def_instrs = &cfg.get_block(*def_node).unwrap().instrs;
            let def = def_instrs
                .iter()
                .find(|instr| instr.get_dest().map_or(false, |d| &d == variable));
            let def_type = def.and_then(|instr| instr.get_type()).unwrap().clone();
            for &node in dom_result
                .dominance_frontier
                .get(&def_node)
                .expect("dominance frontier exists for all nodes")
            {
                // add phi node to block
                if seen
                    .get(variable)
                    .map_or(true, |added| !added.contains(&node))
                {
                    seen.entry(variable)
                        .or_insert_with(|| HashSet::new())
                        .insert(node);
                    let labels = cfg
                        .graph
                        .neighbors_directed(node, petgraph::Direction::Incoming)
                        .sorted()
                        .map(|p| cfg.get_block(p).unwrap().label.to_string())
                        .collect_vec();
                    let args = vec!["undefined".to_string(); labels.len()];
                    let phi = Instruction::Value {
                        dest: variable.to_owned(),
                        op_type: def_type.clone(),
                        op: ValueOps::Phi,
                        args,
                        labels,
                        funcs: Vec::new(),
                        pos: None,
                    };
                    cfg.get_block_mut(node).unwrap().instrs.insert(0, phi);
                }

                // add block to defs
                defs.get_mut(variable)
                    .expect("defs[v] is not empty")
                    .insert(node);
            }
        }
    }
}

struct VariableStack {
    stack: HashMap<String, Vec<String>>,
}

impl VariableStack {
    pub fn new() -> Self {
        VariableStack {
            stack: HashMap::new(),
        }
    }

    pub fn get_last(&self, variable: &String) -> Option<String> {
        self.stack.get(variable).and_then(|v| v.last()).cloned()
    }

    pub fn push(&mut self, variable: &String, value: String) {
        self.stack
            .entry(variable.to_owned())
            .or_default()
            .push(value);
    }

    pub fn pop(&mut self, variable: &String, n: usize) {
        if let Some(v) = self.stack.get_mut(variable) {
            v.truncate(v.len().saturating_sub(n));
        }
    }
}

fn rename(
    node: CFGNode,
    stack: &mut VariableStack,
    cfg: &mut CFG,
    dom_result: &DomResult<CFGNode>,
    variable_counter: &mut HashMap<String, usize>,
) {
    if matches!(node, CFGNode::Return) {
        return;
    }
    let mut to_pop: HashMap<String, usize> = HashMap::new();
    let mut blocks_mut = CFG::split_blocks_mut(&mut cfg.blocks);
    let block = blocks_mut.get_mut(&node).unwrap();
    let block_label = &block.label.clone();
    for instr in &mut block.instrs {
        if let Some(old_args) = instr.get_args() {
            let new_args = old_args
                .iter()
                .map(|arg| stack.get_last(arg).unwrap_or(arg.to_string()))
                .collect();
            instr.set_args(new_args);
        }
        if let Some(old_dest) = instr.get_dest() {
            let count = variable_counter.entry(old_dest.to_owned()).or_default();
            let new_dest = format!("{}.{}", old_dest, count);
            *count += 1;
            instr.set_dest(new_dest.to_owned());
            stack.push(&old_dest, new_dest);
            *to_pop.entry(old_dest).or_default() += 1;
        }
    }

    for succ in cfg.graph.neighbors(node) {
        for phi in blocks_mut
            .get_mut(&succ)
            .map(|b| &mut b.instrs)
            .unwrap_or(&mut Vec::new())
        {
            match phi {
                Instruction::Value {
                    op: ValueOps::Phi,
                    args,
                    labels,
                    dest,
                    ..
                } => {
                    if let (Some(arg_index), Some(name)) = (
                        labels.iter().position(|label| label == block_label),
                        stack.get_last(&dest),
                    ) {
                        args[arg_index] = name;
                    }
                }
                _ => {}
            }
        }
    }

    for next in dom_result.dominator_tree.neighbors(node) {
        rename(next, stack, cfg, dom_result, variable_counter)
    }

    for (variable, n) in &to_pop {
        stack.pop(variable, *n);
    }
}

pub fn convert_to_ssa(cfg: &mut CFG) {
    let dom_result = cfg.find_dominators();
    insert_phi_nodes(cfg, &dom_result);
    let mut stack = VariableStack::new();
    let mut variable_counter = HashMap::new();
    rename(
        CFGNode::Block(0),
        &mut stack,
        cfg,
        &dom_result,
        &mut variable_counter,
    );
}
