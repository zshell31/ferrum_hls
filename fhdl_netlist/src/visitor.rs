mod codegen;
mod dump;
mod reachability;
mod set_names;
pub(crate) mod transform;

use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use codegen::Verilog;
use reachability::Reachability;
use set_names::SetNames;
use transform::Transform;

use self::dump::Dump;
use crate::{
    netlist::{Module, ModuleId, NetList},
    with_id::WithId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Input,
    Output,
}

impl NetList {
    pub fn transform(&mut self) {
        Transform::new(self).run();
    }

    pub fn reachability(&mut self) {
        Reachability::new(self).run();
    }

    pub fn set_names(&mut self) {
        SetNames::new(self).run();
    }

    pub fn synth_verilog_into_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = BufWriter::new(File::create(path)?);
        self.synth_verilog(file)
    }

    #[inline]
    pub fn synth_verilog<W: Write>(&self, writer: W) -> io::Result<()> {
        Verilog::new(self, writer).synth()
    }

    pub fn dump(&self, skip: bool) {
        Dump::new(self, skip).run()
    }

    pub fn dump_by_mod_id(&self, mod_id: ModuleId, skip: bool) {
        let module = self.module(mod_id).map(|module| module.borrow());
        Dump::new(self, skip).visit_module(module.as_deref());
    }

    pub fn dump_mod(&self, module: WithId<ModuleId, &Module>, skip: bool) {
        Dump::new(self, skip).visit_module(module);
    }

    pub fn run_visitors(&mut self) {
        self.transform();
        self.reachability();
        self.set_names();
    }
}
