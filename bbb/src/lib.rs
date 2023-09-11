use bril_rs::{Code, EffectOps, Function, Instruction};

#[derive(Default, Clone, Debug)]
pub struct Block {
    pub label: Option<String>,
    pub instrs: Vec<Instruction>,
}

pub trait ToCode {
    fn to_code(&self) -> Vec<Code>;
}

impl ToCode for Block {
    fn to_code(&self) -> Vec<Code> {
        let mut result = Vec::new();
        if let Some(label) = &self.label {
            result.push(Code::Label {
                label: label.clone(),
                pos: None,
            })
        }
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

fn is_terminator(instr: &Instruction) -> bool {
    matches!(
        instr,
        Instruction::Effect {
            op: EffectOps::Branch | EffectOps::Jump | EffectOps::Return,
            ..
        }
    )
}

pub fn form_blocks(func: &Function) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_block = Block::default();
    for code in &func.instrs {
        match code {
            Code::Label { label, .. } => {
                blocks.push(current_block.clone());
                current_block = Block::default();
                current_block.label = Some(label.clone());
            }
            Code::Instruction(instr) => {
                current_block.instrs.push(instr.clone());
                if is_terminator(instr) {
                    blocks.push(current_block.clone());
                    current_block = Block::default();
                }
            }
        }
    }
    if current_block.label.is_some() || !current_block.instrs.is_empty() {
        blocks.push(current_block);
    }
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
