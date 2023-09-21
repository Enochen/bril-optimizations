use bril_rs::load_program_from_read;
use cfg::{generate_cfg, CFGNode, CFG};
use dom::{DomResult, DominatorUtil};
use itertools::Itertools;
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
        let pretty = self
            .iter()
            .sorted()
            .map(|node| node.pretty_print(cfg))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{ {} }}", pretty)
    }
}

fn print_results(node: CFGNode, cfg: &CFG, results: &HashMap<CFGNode, impl PrettyPrint>) {
    let label = match node {
        CFGNode::Block(i) => &cfg.blocks.get(i).unwrap().label,
        CFGNode::Return => "return",
    };
    println!(
        "[{}]: {}",
        label,
        results
            .get(&node)
            .map_or("N/A".to_string(), |f| f.pretty_print(cfg))
    );
}

fn main() -> io::Result<()> {
    let program = load_program_from_read(io::stdin());
    for function in program.functions {
        let cfg = generate_cfg(&function);

        let DomResult {
            dominators: dominated_by,
            dominated_by: _,
            dominance_frontier,
            immediate_dominator,
        } = cfg.find_dominators();
        println!("Dominators");
        for i in 0..cfg.blocks.len() {
            print_results(CFGNode::Block(i), &cfg, &dominated_by);
        }
        println!("");

        println!("Domination Frontier");
        for i in 0..cfg.blocks.len() {
            print_results(CFGNode::Block(i), &cfg, &dominance_frontier);
        }
        println!("");

        println!("Immediate Dominator");
        for i in 0..cfg.blocks.len() {
            print_results(CFGNode::Block(i), &cfg, &immediate_dominator);
        }
    }
    Ok(())
}
