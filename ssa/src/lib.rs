use std::{
    borrow::BorrowMut,
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use bril_rs::{Instruction, Type, ValueOps};
use cfg::{CFGNode, DataFlowHelpers, CFG};
use dom::{DomResult, DominatorUtil};
use itertools::Itertools;
use petgraph::Direction::Incoming;
use util::SafeAccess;

#[derive(Debug)]
struct Phi {
    canonical: String,
    dest: Option<String>,
    op_type: Type,
    label_args: HashMap<String, String>,
}

impl Phi {
    fn new(canonical: String, op_type: Type, labels: Vec<String>) -> Self {
        Phi {
            canonical,
            dest: None,
            op_type,
            label_args: labels
                .into_iter()
                .map(|label| (label, "undefined".to_string()))
                .collect(),
        }
    }

    fn to_instr(&self) -> Instruction {
        let (labels, args) = self.label_args.clone().into_iter().unzip();
        Instruction::Value {
            op: ValueOps::Phi,
            args,
            labels,
            dest: self.dest.clone().unwrap_or_default(),
            funcs: Vec::new(),
            pos: None,
            op_type: self.op_type.clone(),
        }
    }
}

pub fn convert_to_ssa(source: &CFG) -> CFG {
    let mut cfg = source.clone();
    let dom_result = cfg.find_dominators();
    let mut phi_nodes = make_phi_nodes(&mut cfg, &dom_result);
    let mut stack = VariableStack::new(cfg.args.iter().map(|arg| &arg.name).collect());
    let mut variable_counter = HashMap::new();
    rename(
        CFGNode::Block(0),
        &mut stack,
        &mut cfg,
        &dom_result,
        &mut variable_counter,
        &mut phi_nodes,
    );
    insert_phi_nodes(&mut cfg, &phi_nodes);
    cfg
}

pub fn convert_from_ssa(source: &CFG) -> CFG {
    let mut cfg = source.clone();
    remove_phi_nodes(&mut cfg);
    cfg
}

fn make_phi_nodes(cfg: &mut CFG, dom_result: &DomResult<CFGNode>) -> HashMap<CFGNode, Vec<Phi>> {
    let mut result: HashMap<CFGNode, Vec<Phi>> = HashMap::new();
    let mut seen: HashMap<&String, HashSet<cfg::CFGNode>> = HashMap::new();
    for (variable, def_nodes) in cfg
        .get_defs()
        .clone()
        .iter()
        .sorted_by_key(|x| Reverse(x.0))
    {
        let mut def_stack = def_nodes.into_iter().collect_vec();
        let mut def_type_opt = None;
        let mut i = 0;
        while let Some(def_node) = def_stack.get(i) {
            i += 1;
            let def_block = cfg.get_block(**def_node);
            if def_block.is_none() {
                continue;
            }
            let def_instrs = &def_block.unwrap().instrs;
            let def_type = def_type_opt.get_or_insert_with(|| {
                def_instrs
                    .iter()
                    .find(|instr| instr.get_dest().map_or(false, |d| &d == variable))
                    .and_then(|instr| instr.get_type())
                    .unwrap()
                    .clone()
            });
            for node in dom_result
                .dominance_frontier
                .get(&def_node)
                .expect("dominance frontier exists for all nodes")
            {
                // add phi node to block
                if seen
                    .get(variable)
                    .map_or(true, |added| !added.contains(node))
                {
                    seen.entry(variable)
                        .or_insert_with(|| HashSet::new())
                        .insert(*node);
                    let labels = cfg
                        .graph
                        .neighbors_directed(*node, petgraph::Direction::Incoming)
                        .sorted()
                        .map(|pred| cfg.get_block(pred).unwrap().label.to_string())
                        .collect();
                    // let args = vec!["undefined".to_string(); labels.len()];
                    let phi = Phi::new(variable.to_owned(), def_type.clone(), labels);
                    result.entry(*node).or_default().push(phi);
                    // cfg.get_block_mut(*node).unwrap().instrs.insert(0, phi);
                }

                // add block to defs
                if !def_stack.contains(&node) {
                    def_stack.push(node);
                }
            }
        }
    }
    result
}

struct VariableStack {
    stack: HashMap<String, Vec<String>>,
}

impl VariableStack {
    pub fn new(init: Vec<&String>) -> Self {
        let stack = init
            .into_iter()
            .map(|var| (var.clone(), vec![var.clone()]))
            .collect();
        VariableStack { stack }
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
    phi_nodes: &mut HashMap<CFGNode, Vec<Phi>>,
) {
    if matches!(node, CFGNode::Return) {
        return;
    }
    let mut to_pop: HashMap<String, usize> = HashMap::new();
    let blocks_mut = CFG::split_blocks_mut(&mut cfg.blocks);
    let block = blocks_mut.get(&node).unwrap();
    let block_label = &block.borrow().label.clone();
    if let Some(phis) = phi_nodes.get_mut(&node) {
        for phi in phis {
            let old_dest = phi.canonical.to_owned();
            let count = variable_counter.entry(old_dest.to_owned()).or_default();
            let new_dest = format!("{}.{}", old_dest, count);
            *count += 1;
            phi.dest = Some(new_dest.to_owned());
            stack.push(&old_dest, new_dest);
            *to_pop.entry(old_dest).or_default() += 1;
        }
    }
    for instr in &mut block.borrow_mut().instrs {
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
        if let Some(phis) = phi_nodes.get_mut(&succ) {
            for phi in phis {
                if let Some(name) = stack.get_last(&phi.canonical) {
                    phi.label_args
                        .entry(block_label.to_string())
                        .and_modify(|arg| *arg = name);
                }
            }
        }
    }

    for next in dom_result.dominator_tree.neighbors(node) {
        rename(next, stack, cfg, dom_result, variable_counter, phi_nodes)
    }

    for (variable, n) in &to_pop {
        stack.pop(variable, *n);
    }
}

fn insert_phi_nodes(cfg: &mut CFG, phi_nodes: &HashMap<CFGNode, Vec<Phi>>) {
    for (&node, phis) in phi_nodes {
        if let Some(block) = cfg.get_block_mut(node) {
            for phi in phis {
                block.instrs.insert(0, phi.to_instr());
            }
        }
    }
}

fn create_empty_value(dest: String, const_type: Type) -> Instruction {
    let value = match const_type {
        Type::Int => bril_rs::Literal::Int(i64::default()),
        Type::Bool => bril_rs::Literal::Bool(bool::default()),
        Type::Float => bril_rs::Literal::Float(f64::default()),
        Type::Char => bril_rs::Literal::Char(char::default()),
        Type::Pointer(_) => bril_rs::Literal::Int(i64::default()),
    };
    Instruction::Constant {
        dest,
        const_type,
        op: bril_rs::ConstOps::Const,
        pos: None,
        value,
    }
}

fn remove_phi_nodes(cfg: &mut CFG) {
    let n = cfg.blocks.len();
    let blocks_mut = CFG::split_blocks_mut(&mut cfg.blocks);
    for i in 1..n {
        let node = CFGNode::Block(i);
        let block = blocks_mut.get(&node).unwrap().borrow().clone();
        block.instrs.iter().for_each(|instr| {
            if let Instruction::Value {
                op: ValueOps::Phi,
                args,
                labels,
                dest,
                op_type,
                ..
            } = instr
            {
                for pred in cfg.graph.neighbors_directed(node, Incoming) {
                    let mut pred_block = blocks_mut.get(&pred).unwrap().borrow_mut();
                    let pred_label = pred_block.label.clone();
                    let arg_index = labels
                        .iter()
                        .position(|label| label == &pred_label)
                        .unwrap();
                    let pred_instrs = &mut pred_block.instrs;
                    let arg = args[arg_index].clone();
                    if arg == "undefined" {
                        pred_instrs.insert(
                            pred_instrs.len() - 1,
                            create_empty_value(dest.to_string(), op_type.clone()),
                        );
                        continue;
                    }
                    pred_instrs.insert(
                        pred_instrs.len() - 1,
                        Instruction::Value {
                            dest: dest.to_string(),
                            op_type: op_type.clone(),
                            op: ValueOps::Id,
                            args: vec![arg],
                            funcs: Vec::new(),
                            labels: Vec::new(),
                            pos: None,
                        },
                    )
                }
            }
        });
    }
    blocks_mut.values().for_each(|block| {
        block.borrow_mut().instrs.retain(|instr| {
            !matches!(
                instr,
                Instruction::Value {
                    op: ValueOps::Phi,
                    ..
                }
            )
        });
    })
}
