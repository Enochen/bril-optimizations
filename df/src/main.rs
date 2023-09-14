use bril_rs::load_program_from_read;
use df::{cfg::CFGNode, format_defs, reaching_defs};
use std::io;

use df::cfg::generate_cfg;

fn main() -> io::Result<()> {
    let program = load_program_from_read(io::stdin());
    for function in program.functions {
        let cfg = generate_cfg(&function);
        let df_res = reaching_defs(&cfg);
        for (i, instr) in cfg.blocks.iter().enumerate() {
            println!("{}", &instr.label);
            println!(
                "   in: {}",
                df_res
                    .in_map
                    .get(&CFGNode::Block(i))
                    .map_or("not found".to_string(), |set| format_defs(&set, &cfg))
            );
            println!(
                "   out: {}",
                df_res
                    .out_map
                    .get(&CFGNode::Block(i))
                    .map_or("not found".to_string(), |set| format_defs(&set, &cfg))
            );
        }
    }
    Ok(())
}
