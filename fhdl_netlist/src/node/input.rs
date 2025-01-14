use std::fmt::Debug;

use fhdl_data_structures::graph::NodeId;

use super::{IsNode, MakeNode, NodeOutput};
#[cfg(test)]
use crate::netlist::NodeWithInputs;
use crate::{netlist::Module, node_ty::NodeTy, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlSignalKind {
    None,
    Clk,
    Rst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    pub output: [NodeOutput; 1],
    pub global: GlSignalKind,
}

pub struct InputArgs {
    pub ty: NodeTy,
    pub sym: Option<Symbol>,
}

impl MakeNode<InputArgs> for Input {
    fn make(module: &mut Module, args: InputArgs) -> NodeId {
        let InputArgs { ty, sym } = args;
        module.add_node(Input {
            output: [NodeOutput::wire(ty, sym)],
            global: GlSignalKind::None,
        })
    }
}

#[cfg(test)]
impl NodeWithInputs {
    pub fn input(ty: NodeTy, sym: Option<impl AsRef<str>>, skip: bool) -> Self {
        use std::iter;

        Self::new(
            Input {
                output: [NodeOutput::wire(ty, sym.map(Symbol::intern)).set_skip(skip)],
                global: GlSignalKind::None,
            },
            iter::empty(),
        )
    }
}

impl IsNode for Input {
    #[inline]
    fn in_count(&self) -> usize {
        0
    }

    #[inline]
    fn out_count(&self) -> usize {
        1
    }

    #[inline]
    fn outputs(&self) -> &[NodeOutput] {
        &self.output
    }

    #[inline]
    fn outputs_mut(&mut self) -> &mut [NodeOutput] {
        &mut self.output
    }
}
