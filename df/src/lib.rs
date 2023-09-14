use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use bbb::Block;
use cfg::{CFGNode, CFG};
use petgraph::Direction::Incoming;
use util::SafeAccess;

pub mod cfg;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Definition {
    pub variable: String,
    pub block_index: usize,
}

pub type ReachingDefs = HashSet<Definition>;

pub trait DataFlowDisplay {
    fn generate_string(&self, cfg: &CFG) -> String;
}

impl DataFlowDisplay for ReachingDefs {
    fn generate_string(&self, cfg: &CFG) -> String {
        if self.is_empty() {
            return "∅".to_string();
        }
        let mut var_map = HashMap::new();
        for def in self {
            var_map
                .entry(def.variable.clone())
                .or_insert_with(|| Vec::new())
                .push(def.block_index);
        }
        for blocks in var_map.values_mut() {
            blocks.sort();
        }

        let mut sorted_vars = self
            .iter()
            .map(|def| def.variable.clone())
            .collect::<Vec<_>>();
        sorted_vars.sort();
        sorted_vars
            .into_iter()
            .map(|var| {
                format!(
                    "\"{}\" <- [{}]",
                    var,
                    var_map
                        .get(&var)
                        .unwrap()
                        .iter()
                        .flat_map(|block_index| cfg.blocks.get(*block_index))
                        .map(|block| block.label.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

trait DataFlowHelpers {
    fn get_defs(&self) -> HashSet<String>;
}

impl DataFlowHelpers for Block {
    fn get_defs(&self) -> HashSet<String> {
        self.instrs
            .iter()
            .flat_map(|instr| instr.get_dest())
            .collect()
    }
}

pub struct DataFlowResult<T> {
    pub in_map: HashMap<CFGNode, T>,
    pub out_map: HashMap<CFGNode, T>,
}

/// Returns mapping from CFGNode to its reaching definitions
pub fn reaching_defs(cfg: &CFG) -> DataFlowResult<ReachingDefs> {
    let mut in_map = HashMap::new();
    let mut out_map = HashMap::new();

    let mut worklist: VecDeque<_> = cfg.graph.nodes().collect();

    while let Some(node) = worklist.pop_front() {
        let in_set: ReachingDefs = cfg
            .graph
            .neighbors_directed(node, Incoming)
            .into_iter()
            .flat_map(|p| out_map.get(&p))
            .flatten()
            .cloned()
            .collect();
        in_map.insert(node, in_set.clone());
        if let CFGNode::Block(block_index) = node {
            let block = cfg.blocks.get(block_index).unwrap();
            let new_defs = block.get_defs();
            let mut out_set: HashSet<_> = new_defs
                .iter()
                .cloned()
                .map(|variable| Definition {
                    block_index,
                    variable: variable,
                })
                .collect();
            let kill_set = in_set
                .into_iter()
                .filter(|def| !new_defs.contains(&def.variable));
            out_set.extend(kill_set);
            out_map.insert(node, out_set);
        }
    }
    DataFlowResult { in_map, out_map }
}
