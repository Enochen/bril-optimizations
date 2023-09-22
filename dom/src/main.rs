use bril_rs::load_program;
use cfg::{generate_cfg, CFGNode, CFG};
use dom::{DomResult, DominatorUtil};
use itertools::Itertools;
use petgraph::Direction::Outgoing;
use std::{
    collections::{HashMap, HashSet},
    io,
};

trait PrettyPrint {
    fn pretty_print(&self, cfg: &CFG) -> String;
}

impl PrettyPrint for CFGNode {
    fn pretty_print(&self, cfg: &CFG) -> String {
        match self {
            CFGNode::Block(i) => cfg.blocks[*i].label.clone(),
            CFGNode::Return => "return".to_string(),
        }
    }
}

impl PrettyPrint for HashSet<CFGNode> {
    fn pretty_print(&self, cfg: &CFG) -> String {
        format!(
            "{{ {} }}",
            self.iter()
                .sorted()
                .map(|node| node.pretty_print(cfg))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn print_node(node: CFGNode, result: Option<&impl PrettyPrint>, cfg: &CFG) {
    let label = match node {
        CFGNode::Block(i) => &cfg.blocks.get(i).unwrap().label,
        CFGNode::Return => "return",
    };
    println!(
        "[{}]: {}",
        label,
        result.map_or("N/A".to_string(), |f| f.pretty_print(cfg))
    );
}

fn print_results(results: &HashMap<CFGNode, impl PrettyPrint>, cfg: &CFG) {
    for i in 0..cfg.blocks.len() {
        let node = CFGNode::Block(i);
        print_node(node, results.get(&node), &cfg);
    }
}

fn print_dominators(cfg: &CFG) {
    let DomResult {
        dominators,
        dominated_by: _,
        dominance_frontier,
        immediate_dominator,
        dominator_tree,
    } = cfg.find_dominators();

    println!("Dominators");
    print_results(&dominators, &cfg);
    println!("");

    println!("Domination Frontier");
    print_results(&dominance_frontier, &cfg);
    println!("");

    println!("Immediate Dominator");
    print_results(&immediate_dominator, &cfg);
    println!("");

    println!("Dominator Tree");
    let adj_list: HashMap<_, HashSet<_>> = dominator_tree
        .nodes()
        .map(|n| (n, dominator_tree.neighbors(n).collect()))
        .collect();
    print_results(&adj_list, &cfg);
}

fn main() -> io::Result<()> {
    let program = load_program();
    for function in program.functions {
        let cfg = generate_cfg(&function);
        print_dominators(&cfg);
    }
    Ok(())
}
