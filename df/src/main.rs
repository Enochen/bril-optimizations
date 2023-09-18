use bril_rs::load_program_from_read;
use df::{run_worklist, DataFlowDisplay, DataFlowResult};
use std::{env, io};

use cfg::{generate_cfg, CFGNode, CFG};

use live_vars::LiveVars;
use reaching_defs::ReachingDefs;

mod live_vars;
mod reaching_defs;

enum ResultWrapper {
    ReachingDefs(DataFlowResult<ReachingDefs>),
    LiveVars(DataFlowResult<LiveVars>),
}

impl ResultWrapper {
    fn print_cfg(&self, cfg: &CFG) {
        let dyn_print: Box<dyn Fn(usize)> = match self {
            ResultWrapper::ReachingDefs(res) => Box::new(|i| {
                print_node(CFGNode::Block(i), &cfg, res);
            }),
            ResultWrapper::LiveVars(res) => Box::new(|i| {
                print_node(CFGNode::Block(i), &cfg, res);
            }),
        };
        for i in 0..cfg.blocks.len() {
            dyn_print(i);
        }
    }
}

fn print_node<T: DataFlowDisplay>(node: CFGNode, cfg: &CFG, res: &DataFlowResult<T>) {
    let format = |f: Option<&T>| f.map_or("N/A".to_string(), |set| set.generate_string(&cfg));
    let label = match node {
        CFGNode::Block(i) => &cfg.blocks.get(i).unwrap().label,
        CFGNode::Return => "return",
    };
    println!("[{}]", label);
    println!("   in: {}", format(res.in_map.get(&node)));
    println!("   out: {}", format(res.out_map.get(&node)));
}

fn run_reaching_defs(cfg: &CFG) -> ResultWrapper {
    ResultWrapper::ReachingDefs(run_worklist::<ReachingDefs>(&cfg))
}

fn run_live_vars(cfg: &CFG) -> ResultWrapper {
    ResultWrapper::LiveVars(run_worklist::<LiveVars>(&cfg))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let alg_input = args[1].as_str();
    let run_algorithm = match alg_input {
        "reaching_defs" => run_reaching_defs,
        "live_vars" => run_live_vars,
        _ => {
            eprintln!("Unknown command: {}", alg_input);
            std::process::exit(1);
        }
    };

    let program = load_program_from_read(io::stdin());
    for function in program.functions {
        let cfg = generate_cfg(&function);
        run_algorithm(&cfg).print_cfg(&cfg);
        // print_node(CFGNode::Return, &df_res, &cfg);
    }
    Ok(())
}
