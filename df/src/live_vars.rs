use std::{collections::HashSet, ops::Deref};

use cfg::{CFGNode, CFG};
use df::{Analysis, DataFlowDisplay, DataFlowHelpers, Direction};
use itertools::Itertools;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct LiveVars(HashSet<String>);

impl Deref for LiveVars {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Analysis for LiveVars {
    fn direction() -> Direction {
        Direction::Forward
    }

    fn meet(&self, other: &Self) -> Self {
        LiveVars(self.union(&other).cloned().collect())
    }

    fn transfer(&self, node: &CFGNode, cfg: &CFG) -> Self {
        if let CFGNode::Block(block_index) = node {
            let block = cfg.blocks.get(*block_index).unwrap();
            let mut gen_set = block.get_uses();
            let kill_set = block.get_defs();
            let mut in_set: HashSet<_> = self.iter().cloned().collect();
            in_set.retain(|variable| !kill_set.contains(variable));
            gen_set.extend(in_set);
            return LiveVars(gen_set);
        }
        self.clone()
    }
}

impl DataFlowDisplay for LiveVars {
    fn generate_string(&self, _cfg: &CFG) -> String {
        if self.is_empty() {
            return "∅".to_string();
        }

        self.iter().sorted().cloned().collect::<Vec<_>>().join(" ")
    }
}
