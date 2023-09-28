use bril_rs::{Instruction, Type, ValueOps};
use cfg::{CFGNode, CFG};
use petgraph::Direction::Incoming;

trait DefaultValue {
    fn generate_default(&self, dest: String) -> Instruction;
}

impl DefaultValue for Type {
    fn generate_default(&self, dest: String) -> Instruction {
        let value = match self {
            Type::Int => bril_rs::Literal::Int(i64::default()),
            Type::Bool => bril_rs::Literal::Bool(bool::default()),
            Type::Float => bril_rs::Literal::Float(f64::default()),
            Type::Char => bril_rs::Literal::Char(char::default()),
            Type::Pointer(_) => bril_rs::Literal::Int(i64::default()),
        };
        Instruction::Constant {
            dest,
            const_type: self.to_owned(),
            op: bril_rs::ConstOps::Const,
            pos: None,
            value,
        }
    }
}

pub fn remove_phi_nodes(cfg: &mut CFG) {
    let n = cfg.blocks.len();
    let blocks_mut = CFG::split_blocks_mut(&mut cfg.blocks);
    for i in 1..n {
        let node = CFGNode::Block(i);
        let block = blocks_mut.get(&node).unwrap().borrow().clone();
        block.instrs.iter().for_each(|instr| {
            if let Instruction::Value {
                op: ValueOps::Phi,
                args,
                labels,
                dest,
                op_type,
                ..
            } = instr
            {
                for pred in cfg.graph.neighbors_directed(node, Incoming) {
                    let mut pred_block = blocks_mut.get(&pred).unwrap().borrow_mut();
                    let pred_label = pred_block.label.clone();
                    let arg_index = labels
                        .iter()
                        .position(|label| label == &pred_label)
                        .unwrap();
                    let pred_instrs = &mut pred_block.instrs;
                    let arg = args[arg_index].clone();
                    if arg == "undefined" {
                        pred_instrs.insert(
                            pred_instrs.len() - 1,
                            op_type.generate_default(dest.to_string()),
                        );
                        continue;
                    }
                    pred_instrs.insert(
                        pred_instrs.len() - 1,
                        Instruction::Value {
                            dest: dest.to_string(),
                            op_type: op_type.clone(),
                            op: ValueOps::Id,
                            args: vec![arg],
                            funcs: Vec::new(),
                            labels: Vec::new(),
                            pos: None,
                        },
                    )
                }
            }
        });
    }
    blocks_mut.values().for_each(|block| {
        block.borrow_mut().instrs.retain(|instr| {
            !matches!(
                instr,
                Instruction::Value {
                    op: ValueOps::Phi,
                    ..
                }
            )
        });
    })
}
