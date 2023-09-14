use std::collections::HashMap;

use bbb::{form_blocks, Block};
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
