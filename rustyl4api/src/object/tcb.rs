use crate::object::ObjType;
use crate::error::SysResult;
use crate::syscall::{MsgInfo, SyscallOp, syscall};

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct TcbObj {}

impl KernelObject for TcbObj {
    fn obj_type() -> ObjType { ObjType::Tcb }
}

impl Capability<TcbObj> {
    pub fn configure(&self, vspace_cap: usize, cspace_cap: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbConfigure, 2);
        let mut args = [self.slot, vspace_cap, cspace_cap, 0, 0, 0];
        syscall(info, &mut args).map(|_|())
    }

    pub fn set_registers(&self, flags: usize, elr: usize, sp: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbSetRegisters, 3);
        let mut args = [self.slot, flags, elr, sp, 0, 0];
        syscall(info, &mut args).map(|_|())
    }

    pub fn resume(&self) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbResume, 0);
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        syscall(info, &mut args).map(|_|())
    }
}