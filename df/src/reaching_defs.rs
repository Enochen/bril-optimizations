use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use bbb::BlockHelpers;
use cfg::{CFGNode, CFG};
use df::{Analysis, DataFlowDisplay, Direction};
use itertools::Itertools;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Definition {
    pub variable: String,
    pub block_index: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ReachingDefs(HashSet<Definition>);

impl Deref for ReachingDefs {
    type Target = HashSet<Definition>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Analysis for ReachingDefs {
    fn direction() -> Direction {
        Direction::Forward
    }

    fn meet(&self, other: &Self) -> Self {
        ReachingDefs(self.union(&other).cloned().collect())
    }

    fn transfer(&self, node: &CFGNode, cfg: &CFG) -> Self {
        if let CFGNode::Block(block_index) = node {
            let block = cfg.blocks.get(*block_index).unwrap();
            let new_defs = block.get_defs();
            let mut out_set: HashSet<_> = new_defs
                .iter()
                .cloned()
                .map(|variable| Definition {
                    block_index: *block_index,
                    variable,
                })
                .collect();
            let kill_set = self
                .iter()
                .cloned()
                .filter(|def| !new_defs.contains(&def.variable));
            out_set.extend(kill_set);
            return ReachingDefs(out_set);
        }
        self.clone()
    }
}

impl DataFlowDisplay for ReachingDefs {
    fn generate_string(&self, cfg: &CFG) -> String {
        if self.is_empty() {
            return "∅".to_string();
        }
        let mut var_map = self.iter().fold(HashMap::new(), |mut map, def| {
            map.entry(def.variable.clone())
                .or_insert_with(|| Vec::new())
                .push(def.block_index);
            map
        });
        var_map.values_mut().for_each(|b| b.sort());

        var_map
            .iter()
            .sorted()
            .map(|(var, blocks)| {
                format!(
                    "\"{}\" <- [{}]",
                    var,
                    blocks
                        .iter()
                        .flat_map(|block_index| cfg.blocks.get(*block_index))
                        .map(|block| block.label.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .map(|str| "\n      ".to_string() + &str)
            .collect()
    }
}
