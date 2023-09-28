use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use bril_rs::{Instruction, Type, ValueOps};
use cfg::{CFGNode, DataFlowHelpers, CFG};
use dom::DominatorUtil;
use itertools::Itertools;
use petgraph::{prelude::DiGraphMap, Direction::Incoming};
use util::SafeAccess;

pub fn convert_to_ssa(source: &CFG) -> CFG {
    let dom = source.find_dominators();
    let mut converter = SSAConverter::new(source.clone());
    let mut phi_nodes = converter.make_phi_nodes(&dom.dominance_frontier);
    converter.rename(CFGNode::Block(0), &mut phi_nodes, &dom.dominator_tree);
    converter.insert_phi_nodes(&phi_nodes);
    converter.cfg
}

pub fn convert_from_ssa(source: &CFG) -> CFG {
    let mut cfg = source.clone();
    remove_phi_nodes(&mut cfg);
    cfg
}

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

type PhiResult = HashMap<CFGNode, Vec<Phi>>;

struct SSAConverter {
    cfg: CFG,
    stack: VariableStack,
    counter: VariableCounter,
}

impl SSAConverter {
    fn new(cfg: CFG) -> Self {
        let stack = VariableStack::new(cfg.args.iter().map(|arg| &arg.name).collect());
        SSAConverter {
            cfg,
            stack,
            counter: VariableCounter::new(),
        }
    }

    fn make_phi_nodes(
        &mut self,
        dominance_frontier: &HashMap<CFGNode, HashSet<CFGNode>>,
    ) -> PhiResult {
        let mut result: HashMap<CFGNode, Vec<Phi>> = HashMap::new();
        let mut seen: HashMap<&String, HashSet<cfg::CFGNode>> = HashMap::new();
        for (variable, def_nodes) in self
            .cfg
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
                let def_block = self.cfg.get_block(**def_node);
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
                for node in dominance_frontier
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
                        let labels = self
                            .cfg
                            .graph
                            .neighbors_directed(*node, petgraph::Direction::Incoming)
                            .sorted()
                            .map(|pred| self.cfg.get_block(pred).unwrap().label.to_string())
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

    fn rename(
        &mut self,
        node: CFGNode,
        phi_nodes: &mut PhiResult,
        dominator_tree: &DiGraphMap<CFGNode, ()>,
    ) {
        if matches!(node, CFGNode::Return) {
            return;
        }
        let mut to_pop: HashMap<String, usize> = HashMap::new();
        let blocks_mut = CFG::split_blocks_mut(&mut self.cfg.blocks);
        let block = blocks_mut.get(&node).unwrap();
        let block_label = &block.borrow().label.clone();
        let mut register_name = |old_name: String, stack: &mut VariableStack| {
            let count = self.counter.inc(old_name.to_owned());
            let new_name = format!("{}.{}", old_name, count);
            stack.push(&old_name, new_name.to_string());
            *to_pop.entry(old_name.to_string()).or_default() += 1;
            new_name
        };
        if let Some(phis) = phi_nodes.get_mut(&node) {
            for phi in phis {
                let new_dest = register_name(phi.canonical.to_owned(), &mut self.stack);
                phi.dest = Some(new_dest);
            }
        }
        for instr in &mut block.borrow_mut().instrs {
            if let Some(old_args) = instr.get_args() {
                let new_args = old_args
                    .iter()
                    .map(|arg| self.stack.get_last(arg).unwrap_or(arg.to_string()))
                    .collect();
                instr.set_args(new_args);
            }
            if let Some(old_dest) = instr.get_dest() {
                let new_dest = register_name(old_dest, &mut self.stack);
                instr.set_dest(new_dest);
            }
        }
        for succ in self.cfg.graph.neighbors(node) {
            if let Some(phis) = phi_nodes.get_mut(&succ) {
                for phi in phis {
                    if let Some(name) = self.stack.get_last(&phi.canonical) {
                        phi.label_args
                            .entry(block_label.to_string())
                            .and_modify(|arg| *arg = name);
                    }
                }
            }
        }

        for next in dominator_tree.neighbors(node) {
            self.rename(next, phi_nodes, &dominator_tree);
        }

        for (variable, n) in &to_pop {
            self.stack.pop(variable, *n);
        }
    }

    fn insert_phi_nodes(&mut self, phi_nodes: &HashMap<CFGNode, Vec<Phi>>) {
        for (&node, phis) in phi_nodes {
            if let Some(block) = self.cfg.get_block_mut(node) {
                for phi in phis {
                    block.instrs.insert(0, phi.to_instr());
                }
            }
        }
    }
}

struct VariableStack {
    stack: HashMap<String, Vec<String>>,
}

impl VariableStack {
    fn new(init: Vec<&String>) -> Self {
        let stack = init
            .into_iter()
            .map(|var| (var.clone(), vec![var.clone()]))
            .collect();
        VariableStack { stack }
    }

    fn get_last(&self, variable: &String) -> Option<String> {
        self.stack.get(variable).and_then(|v| v.last()).cloned()
    }

    fn push(&mut self, variable: &String, value: String) {
        self.stack
            .entry(variable.to_owned())
            .or_default()
            .push(value);
    }

    fn pop(&mut self, variable: &String, n: usize) {
        if let Some(v) = self.stack.get_mut(variable) {
            v.truncate(v.len().saturating_sub(n));
        }
    }
}

struct VariableCounter {
    counter: HashMap<String, usize>,
}

impl VariableCounter {
    fn new() -> Self {
        VariableCounter {
            counter: HashMap::new(),
        }
    }

    fn inc(&mut self, variable: String) -> usize {
        let count = self.counter.entry(variable).or_default();
        *count += 1;
        *count - 1
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
