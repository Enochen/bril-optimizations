use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use bbb::{form_blocks, Block, BlockHelpers};
use bril_rs::{Argument, EffectOps, Function, Instruction};
use petgraph::prelude::DiGraphMap;

#[derive(Debug, Clone)]
pub struct CFG {
    pub blocks: Vec<Block>,
    pub graph: DiGraphMap<CFGNode, Edge>,
    pub args: Vec<Argument>,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CFGNode {
    Block(usize),
    Return,
}

#[derive(Debug, Clone)]
pub enum Edge {
    Always,
    Bool(bool),
}

pub trait DataFlowHelpers {
    fn get_defs(&self) -> HashMap<String, HashSet<CFGNode>>;
    fn get_uses(&self) -> HashMap<String, HashSet<CFGNode>>;
}

impl DataFlowHelpers for CFG {
    fn get_defs(&self) -> HashMap<String, HashSet<CFGNode>> {
        let mut result = HashMap::new();
        self.blocks.iter().enumerate().for_each(|(index, block)| {
            block.get_defs().into_iter().for_each(|def| {
                result
                    .entry(def)
                    .or_insert_with(|| HashSet::new())
                    .insert(CFGNode::Block(index));
            })
        });
        result
    }

    fn get_uses(&self) -> HashMap<String, HashSet<CFGNode>> {
        let mut result = HashMap::new();
        self.blocks.iter().enumerate().for_each(|(index, block)| {
            block.get_uses().into_iter().for_each(|r#use| {
                result
                    .entry(r#use)
                    .or_insert_with(|| HashSet::new())
                    .insert(CFGNode::Block(index));
            })
        });
        result
    }
}

impl CFG {
    pub fn get_block(&self, node: CFGNode) -> Option<&Block> {
        match node {
            CFGNode::Block(i) => self.blocks.get(i),
            CFGNode::Return => None,
        }
    }

    pub fn get_block_mut(&mut self, node: CFGNode) -> Option<&mut Block> {
        match node {
            CFGNode::Block(i) => self.blocks.get_mut(i),
            CFGNode::Return => None,
        }
    }

    pub fn split_blocks_mut(blocks: &mut Vec<Block>) -> HashMap<CFGNode, RefCell<&mut Block>> {
        blocks
            .iter_mut()
            .enumerate()
            .map(|(i, b)| (CFGNode::Block(i), RefCell::new(b)))
            .collect()
    }
}

impl Display for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            writeln!(f, "[Block: {}]", block.label)?;
            for instr in &block.instrs {
                writeln!(f, "  {}", instr)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

fn generate_label_index(blocks: &Vec<Block>) -> HashMap<String, usize> {
    blocks
        .iter()
        .enumerate()
        .map(|(i, block)| (block.label.to_owned(), i))
        .collect()
}

pub fn generate_cfg(function: &Function) -> CFG {
    let mut blocks = form_blocks(function);
    let label_index = generate_label_index(&blocks);
    let mut graph = DiGraphMap::new();

    let n = blocks.len();
    for i in 0..n {
        let (left, right) = blocks.split_at_mut(i + 1);
        let block = &mut left[i];
        let node = CFGNode::Block(i);
        graph.add_node(node);
        match block.instrs.last() {
            Some(Instruction::Effect {
                op: EffectOps::Return,
                ..
            }) => {
                graph.add_edge(node, CFGNode::Return, Edge::Always);
            }
            Some(Instruction::Effect {
                op: EffectOps::Jump,
                labels,
                ..
            }) => {
                graph.add_edge(
                    node,
                    CFGNode::Block(*label_index.get(&labels[0]).unwrap()),
                    Edge::Always,
                );
            }
            Some(Instruction::Effect {
                op: EffectOps::Branch,
                labels,
                ..
            }) => {
                graph.add_edge(
                    node,
                    CFGNode::Block(*label_index.get(&labels[0]).unwrap()),
                    Edge::Bool(true),
                );
                graph.add_edge(
                    node,
                    CFGNode::Block(*label_index.get(&labels[1]).unwrap()),
                    Edge::Bool(false),
                );
            }
            _ => {
                if let Some(next) = right.get(0) {
                    graph.add_edge(node, CFGNode::Block(i + 1), Edge::Always);
                    block.instrs.push(Instruction::Effect {
                        op: EffectOps::Jump,
                        labels: vec![next.label.to_owned()],
                        args: Vec::new(),
                        funcs: Vec::new(),
                        pos: None,
                    });
                } else {
                    graph.add_edge(node, CFGNode::Return, Edge::Always);
                    block.instrs.push(Instruction::Effect {
                        op: EffectOps::Return,
                        args: Vec::new(),
                        funcs: Vec::new(),
                        labels: Vec::new(),
                        pos: None,
                    });
                }
            }
        };
    }

    CFG {
        blocks,
        graph,
        args: function.args.clone(),
    }
}
