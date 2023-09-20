use std::collections::{hash_map::Entry, HashMap, HashSet};

use cfg::{CFGNode, CFG};
use petgraph::{
    visit::{DfsPostOrder, IntoNeighbors, Visitable},
    Direction::{Incoming, Outgoing},
};

pub struct DomResult<T> {
    /// For a node, gives set of nodes that it is dominated by (its dominators)
    pub dominated_by: HashMap<T, HashSet<T>>,
    /// For a node, gives set of nodes that it is the dominator of
    pub dominator_of: HashMap<T, HashSet<T>>,
    pub dominance_frontier: HashMap<T, HashSet<T>>,
}

fn reverse_postorder<G, T>(graph: G, start: T) -> Vec<T>
where
    G: IntoNeighbors + Visitable<NodeId = T>,
    T: Copy + PartialEq,
{
    let mut dfs = DfsPostOrder::new(&graph, start);
    let mut result = Vec::new();
    while let Some(next) = dfs.next(&graph) {
        result.push(next);
    }
    result.reverse();
    result
}

pub fn find_dominators(cfg: &CFG) -> DomResult<CFGNode> {
    let all_nodes: HashSet<_> = cfg.graph.nodes().into_iter().collect();

    let mut dominated_by: HashMap<_, _> = all_nodes
        .iter()
        .map(|node| (*node, all_nodes.clone()))
        .collect();

    dominated_by.insert(
        CFGNode::Block(0),
        vec![CFGNode::Block(0)].into_iter().collect(),
    );

    // Navigate in reverse postorder to terminate faster
    let nodes = reverse_postorder(&cfg.graph, CFGNode::Block(0));

    loop {
        let mut changed = false;
        for &node in nodes.iter().skip(1) {
            let prev_doms = dominated_by.get(&node).cloned();
            let mut new_doms = cfg
                .graph
                .neighbors_directed(node, Incoming)
                .map(|n| dominated_by.get(&n).unwrap())
                .cloned()
                .reduce(|a, b| a.intersection(&b).copied().collect::<HashSet<_>>())
                .unwrap();
            new_doms.insert(node);
            let insert_res = dominated_by.insert(node, new_doms);
            if insert_res != prev_doms {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut dominator_of: HashMap<_, HashSet<_>> = HashMap::new();
    for (&node, doms) in &dominated_by {
        for &dom in doms {
            match dominator_of.entry(dom) {
                Entry::Occupied(mut e) => {
                    e.get_mut().insert(node);
                }
                Entry::Vacant(e) => {
                    e.insert(HashSet::new()).insert(node);
                }
            }
        }
    }

    let find_frontier = |doms: &HashSet<_>| {
        doms.iter()
            .flat_map(|dom| cfg.graph.neighbors_directed(*dom, Outgoing))
            .collect::<HashSet<_>>()
            .difference(doms)
            .copied()
            .collect()
    };
    let dominance_frontier = dominator_of
        .iter()
        .map(|(node, doms)| (node.clone(), find_frontier(&doms)))
        .collect();

    DomResult {
        dominated_by,
        dominator_of,
        dominance_frontier,
    }
}
