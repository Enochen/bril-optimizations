use std::io;

use bbb::ToCode;
use bril_rs::{load_program, output_program};
use cfg::generate_cfg;
use ssa::convert_to_ssa;

fn main() -> io::Result<()> {
    let mut program = load_program();
    for function in &mut program.functions {
        let mut cfg = generate_cfg(&function);
        convert_to_ssa(&mut cfg);
        function.instrs = cfg.blocks.to_code();
    }
    output_program(&program);
    Ok(())
}
