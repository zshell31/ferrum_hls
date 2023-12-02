use rustc_macros::{Decodable, Encodable};

use super::{IsNode, NodeKind, NodeOutput};
use crate::{
    net_list::NodeOutId,
    sig_ty::{ConstParam, NodeTy},
    symbol::Symbol,
};

#[derive(Debug, Clone, Copy, Encodable, Decodable)]
pub struct Const {
    pub value: ConstParam,
    pub output: NodeOutput,
}

impl Const {
    pub fn new(ty: NodeTy, value: ConstParam, sym: Option<Symbol>) -> Self {
        Self {
            value,
            output: NodeOutput::wire(ty, sym),
        }
    }
}

impl From<Const> for NodeKind {
    fn from(node: Const) -> Self {
        Self::Const(node)
    }
}

impl IsNode for Const {
    type Inputs = [NodeOutId];
    type Outputs = NodeOutput;

    fn inputs(&self) -> &Self::Inputs {
        &[]
    }

    fn inputs_mut(&mut self) -> &mut Self::Inputs {
        &mut []
    }

    fn outputs(&self) -> &Self::Outputs {
        &self.output
    }

    fn outputs_mut(&mut self) -> &mut Self::Outputs {
        &mut self.output
    }
}

#[derive(Debug, Clone, Encodable, Decodable)]
pub struct MultiConst {
    pub values: Vec<u128>,
    pub outputs: Vec<NodeOutput>,
}

impl MultiConst {
    pub fn new(
        values: impl IntoIterator<Item = u128>,
        outputs: impl IntoIterator<Item = NodeOutput>,
    ) -> Self {
        Self {
            values: values.into_iter().collect(),
            outputs: outputs.into_iter().collect(),
        }
    }
}

impl From<MultiConst> for NodeKind {
    fn from(node: MultiConst) -> Self {
        Self::MultiConst(node)
    }
}

impl IsNode for MultiConst {
    type Inputs = [NodeOutId];
    type Outputs = [NodeOutput];

    fn inputs(&self) -> &Self::Inputs {
        &[]
    }
    fn inputs_mut(&mut self) -> &mut Self::Inputs {
        &mut []
    }

    fn outputs(&self) -> &Self::Outputs {
        self.outputs.as_slice()
    }

    fn outputs_mut(&mut self) -> &mut Self::Outputs {
        self.outputs.as_mut_slice()
    }
}
