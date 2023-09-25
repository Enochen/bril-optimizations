use std::collections::HashSet;

use bril_rs::{Code, EffectOps, Function, Instruction};
use util::SafeAccess;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Block {
    pub label: String,
    pub instrs: Vec<Instruction>,
}

pub trait ToCode {
    fn to_code(&self) -> Vec<Code>;
}

impl ToCode for Block {
    fn to_code(&self) -> Vec<Code> {
        let mut result = Vec::new();
        result.push(Code::Label {
            label: self.label.clone(),
            pos: None,
        });
        for instr in &self.instrs {
            result.push(Code::Instruction(instr.clone()));
        }
        result
    }
}

impl ToCode for Vec<Block> {
    fn to_code(&self) -> Vec<Code> {
        self.iter().flat_map(|b| b.to_code()).collect()
    }
}

pub trait BlockHelpers {
    fn get_defs(&self) -> HashSet<String>;
    fn get_uses(&self) -> HashSet<String>;
}

impl BlockHelpers for Block {
    fn get_defs(&self) -> HashSet<String> {
        self.instrs
            .iter()
            .flat_map(|instr| instr.get_dest())
            .collect()
    }
    fn get_uses(&self) -> HashSet<String> {
        self.instrs
            .iter()
            .flat_map(|instr| instr.get_args())
            .flatten()
            .collect()
    }
}

fn is_terminator(instr: &Instruction) -> bool {
    matches!(
        instr,
        Instruction::Effect {
            op: EffectOps::Branch | EffectOps::Jump | EffectOps::Return,
            ..
        }
    )
}

fn create_unique_label(counter: &mut i32, used_labels: &HashSet<String>) -> String {
    let create_label = |v| format!("anon_block_{}", v);
    let mut label = create_label(*counter);
    while used_labels.contains(&label) {
        *counter += 1;
        label = create_label(*counter);
    }
    label
}

pub fn form_blocks(func: &Function) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_block = Block::default();
    let mut used_labels = HashSet::new();
    for code in &func.instrs {
        match code {
            Code::Label { label, .. } => {
                if current_block != Block::default() {
                    blocks.push(current_block);
                    current_block = Block::default();
                }
                current_block.label = label.clone();
                used_labels.insert(label.clone());
            }
            Code::Instruction(instr) => {
                current_block.instrs.push(instr.clone());
                if is_terminator(instr) {
                    blocks.push(current_block);
                    current_block = Block::default();
                }
            }
        }
    }
    if current_block != Block::default() {
        blocks.push(current_block);
    }
    let mut counter = 0;
    blocks
        .iter_mut()
        .filter(|block| block.label.is_empty())
        .for_each(|block| {
            block.label = create_unique_label(&mut counter, &used_labels);
            used_labels.insert(block.label.clone());
        });
    return blocks;
}
