use std::{collections::HashSet, ops::Deref};

use cfg::CFG;
use df::{Analysis, DataFlowDisplay, DataFlowHelpers};
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
    fn meet(&self, other: &Self) -> Self {
        LiveVars(self.union(&other).cloned().collect())
    }

    fn transfer(&self, block_index: usize, cfg: &CFG) -> Self {
        let block = cfg.blocks.get(block_index).unwrap();
        let defs = block.get_defs();
        let uses = block.get_uses();
        let mut out_set: HashSet<_> = uses.iter().cloned().collect();

        let kill_set = self
            .iter()
            .cloned()
            .filter(|variable| !defs.contains(variable));
        out_set.extend(kill_set);
        LiveVars(out_set)
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
