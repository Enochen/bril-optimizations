use std::collections::{hash_map::Entry, HashMap, HashSet};

use cfg::{CFGNode, CFG};
use petgraph::{
    algo::all_simple_paths,
    prelude::DiGraphMap,
    visit::{DfsPostOrder, IntoNeighbors, Visitable},
    Direction::{Incoming, Outgoing},
};

pub struct DomResult<T> {
    /// dominators[x] is set of nodes that are dominators of x
    pub dominators: HashMap<T, HashSet<T>>,
    /// dominated_by[x] is set of nodes that are dominated by x
    pub dominated_by: HashMap<T, HashSet<T>>,
    /// dominated_by[x] is set of nodes that are on the dominance frontier of x
    pub dominance_frontier: HashMap<T, HashSet<T>>,
    /// immediate_dominator[x] is the immediate dominator of x
    pub immediate_dominator: HashMap<T, T>,
    /// dominator_tree contain nodes whose parent-child relationship is immediate dominance
    pub dominator_tree: DiGraphMap<T, ()>,
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

pub trait DominatorUtil {
    fn find_dominators(&self) -> DomResult<CFGNode>;
}

impl DominatorUtil for CFG {
    fn find_dominators(&self) -> DomResult<CFGNode> {
        let all_nodes: HashSet<_> = self.graph.nodes().into_iter().collect();

        let mut dominators: HashMap<_, _> = all_nodes
            .iter()
            .map(|node| (*node, all_nodes.clone()))
            .collect();

        dominators.insert(
            CFGNode::Block(0),
            vec![CFGNode::Block(0)].into_iter().collect(),
        );

        // Navigate in reverse postorder to terminate faster
        let reverse_postorder_nodes = reverse_postorder(&self.graph, CFGNode::Block(0));

        loop {
            let mut changed = false;
            for &node in reverse_postorder_nodes.iter().skip(1) {
                let prev_doms = dominators.get(&node).cloned();
                let mut new_doms = self
                    .graph
                    .neighbors_directed(node, Incoming)
                    .map(|n| dominators.get(&n).unwrap())
                    .cloned()
                    .reduce(|a, b| a.intersection(&b).copied().collect::<HashSet<_>>())
                    .unwrap();
                new_doms.insert(node);
                let insert_res = dominators.insert(node, new_doms);
                if insert_res != prev_doms {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut dominated_by: HashMap<_, HashSet<_>> = HashMap::new();
        for (&node, doms) in &dominators {
            for &dom in doms {
                match dominated_by.entry(dom) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().insert(node);
                    }
                    Entry::Vacant(e) => {
                        e.insert(HashSet::new()).insert(node);
                    }
                }
            }
        }

        let find_frontier = |node: &CFGNode| {
            let subs = dominated_by
                .get(node)
                .expect("all nodes should exist as keys in dominated_by");
            let strict_subs = subs.difference(&HashSet::from([*node])).copied().collect();
            subs.iter()
                .flat_map(|dom| self.graph.neighbors_directed(*dom, Outgoing))
                .collect::<HashSet<_>>()
                .difference(&strict_subs)
                .copied()
                .collect()
        };

        let dominance_frontier = all_nodes
            .iter()
            .copied()
            .map(|node| (node, find_frontier(&node)))
            .collect();

        let find_immediate_dom = |node: &CFGNode| {
            let doms = dominators
                .get(node)
                .expect("all nodes should exist as keys in dominators");
            doms.iter().filter(|&d| d != node).find(|&d| {
                let candidate_doms = dominated_by
                    .get(d)
                    .expect("all nodes should exist as keys in dominators");
                let intersection: HashSet<_> = candidate_doms.intersection(doms).collect();
                // The immediate dominator will only have two things in this intersection: the dominator and the dominated node
                intersection.len() == 2
            })
        };

        let mut immediate_dominator = HashMap::new();
        for &node in &all_nodes {
            if let Some(&dom) = find_immediate_dom(&node) {
                immediate_dominator.insert(node, dom);
            }
        }

        let mut dominator_tree = DiGraphMap::new();
        dominator_tree.add_node(CFGNode::Block(0));
        for (&node, &imm_dom) in &immediate_dominator {
            dominator_tree.add_edge(imm_dom, node, ());
        }

        for node in all_nodes {
            verify_dominators(&node, dominators.get(&node).unwrap(), &self);
        }

        DomResult {
            dominators,
            dominated_by,
            dominance_frontier,
            immediate_dominator,
            dominator_tree,
        }
    }
}

/// Panics if input dominators does not match actual dominators of input node
fn verify_dominators(node: &CFGNode, dominators: &HashSet<CFGNode>, cfg: &CFG) {
    let naive_doms_opt =
        all_simple_paths::<HashSet<_>, _>(&cfg.graph, CFGNode::Block(0), *node, 0, None)
            .reduce(|a, b| a.intersection(&b).copied().collect());
    // None means node is unreachable from entry
    if let Some(naive_doms) = naive_doms_opt {
        if naive_doms != *dominators {
            panic!("Efficient algorithm calculates {:?}'s dominators is {:?}, but naive algorithm thinks dominators is {:?}", node, dominators, naive_doms)
        }
    }
}
