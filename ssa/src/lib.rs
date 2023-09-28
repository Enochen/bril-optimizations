use bril2ssa::SSAConverter;
use cfg::{CFGNode, CFG};
use dom::DominatorUtil;
use ssa2bril::remove_phi_nodes;

mod bril2ssa;
mod ssa2bril;

pub fn convert_to_ssa(source: &CFG) -> CFG {
    let dom = source.find_dominators();
    let mut converter = SSAConverter::new(source.clone());
    let mut phi_nodes = converter.make_phi_nodes(&dom.dominance_frontier);
    converter.rename(CFGNode::Block(0), &mut phi_nodes, &dom.dominator_tree);
    converter.insert_phi_nodes(&phi_nodes);
    converter.cfg
}

pub fn convert_from_ssa(source: &CFG) -> CFG {
    let mut cfg = source.clone();
    remove_phi_nodes(&mut cfg);
    cfg
}
