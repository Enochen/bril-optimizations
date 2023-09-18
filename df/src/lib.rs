use std::collections::{HashMap, HashSet, VecDeque};

use bbb::Block;
use cfg::{CFGNode, CFG};
use petgraph::EdgeDirection::{self, Incoming, Outgoing};
use util::SafeAccess;

pub trait Analysis: Default + Clone + PartialEq + DataFlowDisplay {
    fn direction() -> Direction;

    fn meet(&self, other: &Self) -> Self;

    fn transfer(&self, node: &CFGNode, cfg: &CFG) -> Self;
}

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

pub enum Direction {
    Forward,
    Backward,
}

impl Into<EdgeDirection> for Direction {
    fn into(self) -> EdgeDirection {
        match self {
            Direction::Forward => Incoming,
            Direction::Backward => Outgoing,
        }
    }
}

pub struct DataFlowResult<T: DataFlowDisplay> {
    pub in_map: HashMap<CFGNode, T>,
    pub out_map: HashMap<CFGNode, T>,
}

/// Returns mapping from CFGNode to its reaching definitions
pub fn run_worklist<T: Analysis>(cfg: &CFG) -> DataFlowResult<T> {
    let mut in_map = HashMap::new();
    let mut out_map = HashMap::new();

    let mut worklist: VecDeque<_> = cfg.graph.nodes().collect();

    let graph_direction = T::direction().into();

    while let Some(node) = worklist.pop_front() {
        let in_set = cfg
            .graph
            .neighbors_directed(node, graph_direction)
            .into_iter()
            .flat_map(|p| out_map.get(&p))
            .fold(T::default(), |acc, next| acc.meet(next));
        in_map.insert(node, in_set.clone());
        let out_set = in_set.transfer(&node, &cfg);
        let old_out = out_map.insert(node, out_set.clone());
        if old_out != Some(out_set) {
            cfg.graph
                .neighbors_directed(node, graph_direction.opposite())
                .for_each(|s| worklist.push_back(s));
        }
    }
    // Swap in and out when going backwards
    if graph_direction == Outgoing {
        (out_map, in_map) = (in_map, out_map)
    }
    DataFlowResult { in_map, out_map }
}
