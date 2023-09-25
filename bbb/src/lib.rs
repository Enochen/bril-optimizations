use std::collections::{HashMap, HashSet};

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

pub trait FunctionHelpers {
    fn get_defs(&self) -> HashMap<String, HashSet<usize>>;
    fn get_uses(&self) -> HashMap<String, HashSet<usize>>;
}

impl FunctionHelpers for Vec<Block> {
    fn get_defs(&self) -> HashMap<String, HashSet<usize>> {
        let mut result = HashMap::new();
        self.iter().enumerate().for_each(|(index, block)| {
            block.get_defs().into_iter().for_each(|def| {
                result
                    .entry(def)
                    .or_insert_with(|| HashSet::new())
                    .insert(index);
            })
        });
        result
    }

    fn get_uses(&self) -> HashMap<String, HashSet<usize>> {
        let mut result = HashMap::new();
        self.iter().enumerate().for_each(|(index, block)| {
            block.get_uses().into_iter().for_each(|r#use| {
                result
                    .entry(r#use)
                    .or_insert_with(|| HashSet::new())
                    .insert(index);
            })
        });
        result
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

fn fold_terms<F>(terms: &Vec<String>, transform: F) -> String
where
    F: Fn(&String) -> String,
{
    terms.iter().map(transform).collect::<Vec<_>>().join(" ")
}

fn format_rhs(op: String, funcs: &Vec<String>, args: &Vec<String>, labels: &Vec<String>) -> String {
    vec![
        op,
        fold_terms(funcs, |s| format!("@{s}")),
        fold_terms(args, |s| format!("{s}")),
        fold_terms(labels, |s| format!(".{s}")),
    ]
    .into_iter()
    .filter(|s| s.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn instr_to_txt(instr: &Instruction) -> String {
    match instr {
        Instruction::Constant {
            dest,
            const_type,
            value,
            ..
        } => format!("{}: {} = const {}", dest, const_type, value),
        Instruction::Value {
            dest,
            op_type,
            op,
            funcs,
            args,
            labels,
            ..
        } => format!(
            "{}: {} = {}",
            dest,
            op_type,
            format_rhs(op.to_string(), funcs, args, labels)
        ),
        Instruction::Effect {
            op,
            funcs,
            args,
            labels,
            ..
        } => format!("{}", format_rhs(op.to_string(), funcs, args, labels)),
    }
}
