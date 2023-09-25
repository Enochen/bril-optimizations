use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use bbb::{form_blocks, Block, BlockHelpers};
use bril_rs::{EffectOps, Function, Instruction};
use petgraph::prelude::DiGraphMap;

#[derive(Debug)]
pub struct CFG {
    pub blocks: Vec<Block>,
    pub graph: DiGraphMap<CFGNode, Edge>,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CFGNode {
    Block(usize),
    Return,
}

#[derive(Debug)]
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
    pub fn get_block(&self, node: CFGNode) -> &Block {
        match node {
            CFGNode::Block(i) => self.blocks.get(i).unwrap(),
            CFGNode::Return => self.blocks.last().unwrap(),
        }
    }

    pub fn get_block_mut(&mut self, node: CFGNode) -> &mut Block {
        match node {
            CFGNode::Block(i) => self.blocks.get_mut(i).unwrap(),
            CFGNode::Return => self.blocks.last_mut().unwrap(),
        }
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
    let blocks = form_blocks(function);
    let label_index = generate_label_index(&blocks);
    let mut graph = DiGraphMap::new();

    for (i, block) in blocks.iter().enumerate() {
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
                if let Some(_) = blocks.get(i + 1) {
                    graph.add_edge(node, CFGNode::Block(i + 1), Edge::Always);
                } else {
                    graph.add_edge(node, CFGNode::Return, Edge::Always);
                }
            }
        };
    }

    CFG { blocks, graph }
}
