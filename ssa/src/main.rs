use std::io;

use bril_rs::load_program;
use cfg::generate_cfg;
use ssa::insert_phi_nodes;

fn main() -> io::Result<()> {
    let program = load_program();
    for function in program.functions {
        let mut cfg = generate_cfg(&function);
        insert_phi_nodes(&mut cfg);
        println!("{}", cfg);
    }
    Ok(())
}
