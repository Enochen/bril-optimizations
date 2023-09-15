use std::collections::{HashMap, HashSet, VecDeque};

use bbb::Block;
use cfg::{CFGNode, CFG};
use petgraph::Direction::{self, Incoming, Outgoing};
use util::SafeAccess;

pub trait DataFlowDisplay {
    fn generate_string(&self, cfg: &CFG) -> String;
}

pub trait DataFlowHelpers {
    fn get_defs(&self) -> HashSet<String>;
    fn get_uses(&self) -> HashSet<String>;
}

impl DataFlowHelpers for Block {
    fn get_defs(&self) -> HashSet<String> {
        self.instrs
            .iter()
            .flat_map(|instr| instr.get_dest())
            .collect()
    }
    fn get_uses(&self) -> HashSet<String> {
        self.instrs
            .iter()
            .flat_map(|instr| instr.get_args())
            .flatten()
            .collect()
    }
}

pub trait Analysis: Default + Clone + PartialEq {
    fn meet(&self, other: &Self) -> Self;

    fn transfer(&self, block_index: usize, cfg: &CFG) -> Self;
}

pub struct DataFlowResult<T> {
    pub in_map: HashMap<CFGNode, T>,
    pub out_map: HashMap<CFGNode, T>,
}

/// Returns mapping from CFGNode to its reaching definitions
pub fn run_worklist<T: Analysis>(cfg: &CFG, direction: Direction) -> DataFlowResult<T> {
    let mut in_map = HashMap::new();
    let mut out_map: HashMap<CFGNode, T> = HashMap::new();

    let mut worklist: VecDeque<_> = cfg.graph.nodes().collect();

    while let Some(node) = worklist.pop_front() {
        let in_set = cfg
            .graph
            .neighbors_directed(node, direction)
            .into_iter()
            .flat_map(|p| out_map.get(&p))
            .fold(T::default(), |acc, next| acc.meet(next));
        in_map.insert(node, in_set.clone());
        if let CFGNode::Block(block_index) = node {
            let out_set = in_set.transfer(block_index, &cfg);
            let old_out = out_map.insert(node, out_set.clone());
            if old_out != Some(out_set) {
                cfg.graph
                    .neighbors_directed(node, direction.opposite())
                    .for_each(|s| worklist.push_back(s));
            }
        }
    }
    match direction {
        Incoming => DataFlowResult { in_map, out_map },
        Outgoing => DataFlowResult {
            in_map: out_map,
            out_map: in_map,
        },
    }
}
