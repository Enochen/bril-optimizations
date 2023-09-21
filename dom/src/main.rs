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

fn print_node(node: CFGNode, cfg: &CFG, result: Option<&impl PrettyPrint>) {
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

fn print_cfg(cfg: &CFG, results: &HashMap<CFGNode, impl PrettyPrint>) {
    for i in 0..cfg.blocks.len() {
        let node = CFGNode::Block(i);
        print_node(node, &cfg, results.get(&node));
    }
}

fn main() -> io::Result<()> {
    let program = load_program();
    for function in program.functions {
        let cfg = generate_cfg(&function);

        let DomResult {
            dominators,
            dominated_by: _,
            dominance_frontier,
            immediate_dominator,
            dominator_tree,
        } = cfg.find_dominators();

        println!("Dominators");
        print_cfg(&cfg, &dominators);
        println!("");

        println!("Domination Frontier");
        print_cfg(&cfg, &dominance_frontier);
        println!("");

        println!("Immediate Dominator");
        print_cfg(&cfg, &immediate_dominator);
        println!("");

        println!("Dominator Tree");
        let adj_list: HashMap<_, HashSet<_>> = dominator_tree
            .nodes()
            .map(|n| (n, dominator_tree.neighbors(n).collect()))
            .collect();
        print_cfg(&cfg, &adj_list);
    }
    Ok(())
}
