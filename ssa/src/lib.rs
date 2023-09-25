use std::collections::{HashMap, HashSet};

use bril_rs::ValueOps;
use cfg::{DataFlowHelpers, CFG};
use dom::{DomResult, DominatorUtil};
use itertools::Itertools;
use util::SafeAccess;

pub fn insert_phi_nodes(cfg: &mut CFG) {
    let mut defs = cfg.get_defs();
    let mut seen: HashMap<&String, HashSet<cfg::CFGNode>> = HashMap::new();
    let DomResult {
        dominance_frontier, ..
    } = cfg.find_dominators();
    for (variable, def_nodes) in &defs.clone() {
        for def_node in def_nodes {
            for &node in dominance_frontier
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
                    let op_type = cfg
                        .get_block(*def_node)
                        .instrs
                        .iter()
                        .find(|instr| instr.get_dest().map_or(false, |d| &d == variable))
                        .and_then(|instr| instr.get_type())
                        .unwrap()
                        .clone();
                    let phi = bril_rs::Instruction::Value {
                        args: Vec::new(),
                        dest: variable.to_owned(),
                        funcs: Vec::new(),
                        labels: cfg
                            .graph
                            .neighbors_directed(node, petgraph::Direction::Incoming)
                            .sorted()
                            .map(|p| cfg.get_block(p).label.to_string())
                            .collect(),
                        op: ValueOps::Phi,
                        pos: None,
                        op_type,
                    };
                    cfg.get_block_mut(node).instrs.insert(0, phi);
                }

                // add block to defs
                defs.get_mut(variable)
                    .expect("defs[v] is not empty")
                    .insert(node);
            }
        }
    }
}
