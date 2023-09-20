use bril_rs::load_program_from_read;
use cfg::{generate_cfg, CFGNode, CFG};
use dom::{find_dominators, DomResult};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    io,
};

fn print_sets(node: CFGNode, cfg: &CFG, sets: &HashMap<CFGNode, HashSet<CFGNode>>) {
    let format = |f: Option<&HashSet<CFGNode>>| {
        let pretty = f.map_or("N/A".to_string(), |set| {
            set.into_iter()
                .sorted()
                .map(|node| match node {
                    CFGNode::Block(i) => cfg.blocks[*i].label.clone(),
                    CFGNode::Return => "Return".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        });
        format!("{{ {} }}", pretty)
    };
    let label = match node {
        CFGNode::Block(i) => &cfg.blocks.get(i).unwrap().label,
        CFGNode::Return => "return",
    };
    println!("[{}]: {}", label, format(sets.get(&node)));
}

fn main() -> io::Result<()> {
    let program = load_program_from_read(io::stdin());
    for function in program.functions {
        let cfg = generate_cfg(&function);

        let DomResult {
            dominated_by,
            dominator_of: _,
            dominance_frontier,
        } = find_dominators(&cfg);
        println!("Dominators");
        for i in 0..cfg.blocks.len() {
            print_sets(CFGNode::Block(i), &cfg, &dominated_by);
        }
        println!("");

        println!("Domination Frontier");
        for i in 0..cfg.blocks.len() {
            print_sets(CFGNode::Block(i), &cfg, &dominance_frontier);
        }
    }
    Ok(())
}
