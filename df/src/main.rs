use bril_rs::load_program_from_read;
use df::{run_worklist, DataFlowDisplay, DataFlowResult};
use reaching_defs::ReachingDefs;
use std::io;

use cfg::{generate_cfg, CFGNode, CFG};

mod reaching_defs;

fn print_node<T: DataFlowDisplay>(node: CFGNode, res: &DataFlowResult<T>, cfg: &CFG) {
    let format = |f: Option<&T>| f.map_or("N/A".to_string(), |set| set.generate_string(&cfg));
    let label = match node {
        CFGNode::Block(i) => &cfg.blocks.get(i).unwrap().label,
        CFGNode::Return => "return",
    };
    println!("[Block: {}]", label);
    println!("   in: {}", format(res.in_map.get(&node)));
    println!("   out: {}", format(res.out_map.get(&node)));
}

fn main() -> io::Result<()> {
    let program = load_program_from_read(io::stdin());
    for function in program.functions {
        let cfg = generate_cfg(&function);
        let df_res = run_worklist::<ReachingDefs>(&cfg);
        for i in 0..cfg.blocks.len() {
            print_node(CFGNode::Block(i), &df_res, &cfg);
        }
        print_node(CFGNode::Return, &df_res, &cfg);
    }
    Ok(())
}
