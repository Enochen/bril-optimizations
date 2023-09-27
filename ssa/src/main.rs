use std::{env, io};

use bbb::ToCode;
use bril_rs::{load_program, output_program};
use cfg::generate_cfg;
use ssa::{convert_from_ssa, convert_to_ssa};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // Valid values: ["into", "full"], defaults to "full"
    let mode = args.get(1).map(|a| a.as_str()).unwrap_or("full");

    let mut program = load_program();
    for function in &mut program.functions {
        let cfg = generate_cfg(&function);
        let ssa_cfg = convert_to_ssa(&cfg);
        if mode == "into" {
            function.instrs = ssa_cfg.blocks.to_code();
            continue;
        }
        if mode == "full" {
            let out_cfg = convert_from_ssa(&ssa_cfg);
            function.instrs = out_cfg.blocks.to_code();
        }
    }
    output_program(&program);
    Ok(())
}
