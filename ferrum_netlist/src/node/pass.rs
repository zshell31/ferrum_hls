use ferrum::prim_ty::PrimTy;

use super::{IsNode, Node, NodeOutput};
use crate::{net_kind::NetKind, net_list::NodeOutId, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct PassNode {
    pub inject: bool,
    pub input: NodeOutId,
    pub output: NodeOutput,
}

impl PassNode {
    pub fn new(ty: PrimTy, input: NodeOutId, sym: Symbol) -> Self {
        Self {
            inject: false,
            input,
            output: NodeOutput {
                ty,
                sym,
                kind: NetKind::Wire,
            },
        }
    }
}

impl From<PassNode> for Node {
    fn from(node: PassNode) -> Self {
        Self::Pass(node)
    }
}

impl IsNode for PassNode {
    type Inputs = NodeOutId;
    type Outputs = NodeOutput;

    fn inputs(&self) -> &Self::Inputs {
        &self.input
    }

    fn outputs(&self) -> &Self::Outputs {
        &self.output
    }

    fn outputs_mut(&mut self) -> &mut Self::Outputs {
        &mut self.output
    }
}
