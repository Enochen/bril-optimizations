use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use cfg::CFG;
use df::{Analysis, DataFlowDisplay, DataFlowHelpers};
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
    fn meet(&self, other: &Self) -> Self {
        ReachingDefs(self.union(&other).cloned().collect())
    }

    fn transfer(&self, block_index: usize, cfg: &CFG) -> Self {
        let block = cfg.blocks.get(block_index).unwrap();
        let new_defs = block.get_defs();
        let mut out_set: HashSet<_> = new_defs
            .iter()
            .cloned()
            .map(|variable| Definition {
                block_index,
                variable,
            })
            .collect();
        let kill_set = self
            .iter()
            .filter(|def| !new_defs.contains(&def.variable))
            .cloned();
        out_set.extend(kill_set);
        ReachingDefs(out_set)
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

        let format_var = |var| {
            var_map.get(&var).map(|blocks| {
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
        };

        self.iter()
            .map(|def| def.variable.clone())
            .sorted()
            .flat_map(format_var)
            .map(|str| "\n      ".to_string() + &str)
            .collect()
    }
}
