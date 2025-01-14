mod module;

use std::{cell::RefCell, ops::Index};

use fhdl_data_structures::{
    graph::NodeId, index::IndexType, index_storage::IndexStorage,
};
#[cfg(test)]
pub(crate) use module::NodeWithInputs;
pub use module::{Incoming, Module, NodeCursor, Outgoing};

pub use self::module::ModuleId;
use crate::{cfg::NetListCfg, with_id::WithId};

#[derive(Debug, Default)]
pub struct NetList {
    pub top: Option<ModuleId>,
    modules: IndexStorage<ModuleId, RefCell<Module>>,
    cfg: NetListCfg,
}

impl Index<ModuleId> for NetList {
    type Output = RefCell<Module>;

    fn index(&self, index: ModuleId) -> &Self::Output {
        &self.modules[index]
    }
}

impl NetList {
    pub fn new(cfg: NetListCfg) -> Self {
        Self {
            top: None,
            modules: Default::default(),
            cfg,
        }
    }

    pub fn cfg(&self) -> &NetListCfg {
        &self.cfg
    }

    #[inline]
    pub fn add_module(&mut self, module: Module) -> ModuleId {
        let mod_id = self.modules.last_idx();

        if module.is_top {
            self.top = Some(mod_id);
        }
        self.modules.push(RefCell::new(module))
    }

    #[inline]
    pub fn module_ids(&mut self) -> impl DoubleEndedIterator<Item = ModuleId> {
        (0 .. self.modules.len()).map(ModuleId::from_usize)
    }

    pub fn modules(
        &self,
    ) -> impl DoubleEndedIterator<Item = WithId<ModuleId, &RefCell<Module>>> + '_ {
        self.modules
            .iter_with_id()
            .map(|(id, inner)| WithId::new(id, inner))
    }

    #[inline]
    pub fn module(&self, module_id: ModuleId) -> WithId<ModuleId, &RefCell<Module>> {
        WithId {
            id: module_id,
            inner: &self.modules[module_id],
        }
    }

    pub fn inline_mod(
        &self,
        mut target_mod: WithId<ModuleId, &mut Module>,
        mod_inst_id: NodeId,
    ) -> Option<NodeId> {
        if let Some(mod_inst) = target_mod.node(mod_inst_id).mod_inst() {
            if mod_inst.mod_id == target_mod.id {
                return None;
            }

            let source_mod = self.module(mod_inst.mod_id).map(|module| module.borrow());
            target_mod.inline_mod(mod_inst_id, source_mod.as_deref())
        } else {
            None
        }
    }
}
