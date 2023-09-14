use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use bbb::Block;
use cfg::{CFGNode, CFG};
use petgraph::Direction::Incoming;
use util::SafeAccess;

pub mod cfg;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Definition {
    pub block_index: usize,
    pub variable: String,
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hi")?;
        Ok(())
    }
}

pub type ReachingDefs = HashSet<Definition>;

pub fn format_defs(defs: &ReachingDefs, cfg: &CFG) -> String {
    let mut sorted_defs = defs.iter().collect::<Vec<_>>();
    sorted_defs.sort();
    sorted_defs
        .into_iter()
        .map(|def| {
            format!(
                "\"{}\" from {}",
                def.variable,
                cfg.blocks.get(def.block_index).unwrap().label
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
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
                .filter(|def| new_defs.contains(&def.variable));
            out_set.extend(kill_set);
            out_map.insert(node, out_set);
        }
    }
    DataFlowResult { in_map, out_map }
}
