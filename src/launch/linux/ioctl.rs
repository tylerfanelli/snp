// SPDX-License-Identifier: Apache-2.0

use super::*;

use crate::{
    error::{Error, Indeterminate},
    impl_const_id,
};

use std::{
    marker::PhantomData,
    os::{raw::c_ulong, unix::io::AsRawFd},
};

use iocuddle::*;

// These enum ordinal values are defined in the Linux kernel
// source code: include/uapi/linux/kvm.h
impl_const_id! {
    /// The ioctl sub number
    pub Id => u32;

    Init = 22,
    LaunchStart<'_> = 23,
    LaunchUpdate<'_> = 24,
    LaunchFinish<'_> = 25,
}

const KVM: Group = Group::new(0xAE);
const ENC_OP: Ioctl<WriteRead, &c_ulong> = unsafe { KVM.write_read(0xBA) };

/// Corresponds to the `KVM_MEMORY_ENCRYPT_REG_REGION` ioctl
pub const ENC_REG_REGION: Ioctl<Write, &KvmEncRegion> =
    unsafe { KVM.read::<KvmEncRegion>(0xBB).lie() };

/// Initialize the SEV-SNP platform in KVM.
pub const INIT: Ioctl<WriteRead, &Command<Init>> = unsafe { ENC_OP.lie() };

/// Initialize the flow to launch a guest.
pub const LAUNCH_START: Ioctl<WriteRead, &Command<LaunchStart>> = unsafe { ENC_OP.lie() };

/// Insert pages into the guest physical address space.
pub const LAUNCH_UPDATE: Ioctl<WriteRead, &Command<LaunchUpdate>> = unsafe { ENC_OP.lie() };

/// Complete the guest launch flow.
pub const LAUNCH_FINISH: Ioctl<WriteRead, &Command<LaunchFinish>> = unsafe { ENC_OP.lie() };

/// Corresponds to the kernel struct `kvm_enc_region`
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct KvmEncRegion<'a> {
    addr: u64,
    size: u64,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> KvmEncRegion<'a> {
    /// Create a new `KvmEncRegion` referencing some memory assigned to the virtual machine.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            addr: data.as_ptr() as _,
            size: data.len() as _,
            phantom: PhantomData,
        }
    }

    /// Register the encrypted memory region to a virtual machine
    pub fn register(&mut self, vm_fd: &mut impl AsRawFd) -> std::io::Result<std::os::raw::c_uint> {
        ENC_REG_REGION.ioctl(vm_fd, self)
    }
}

/// A generic SEV command
#[repr(C)]
pub struct Command<'a, T: Id> {
    code: u32,
    data: u64,
    error: u32,
    sev_fd: u32,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: Id> Command<'a, T> {
    /// create the command from a mutable subcommand
    pub fn from_mut(sev: &'a mut impl AsRawFd, subcmd: &'a mut T) -> Self {
        Self {
            code: T::ID,
            data: subcmd as *mut T as _,
            error: 0,
            sev_fd: sev.as_raw_fd() as _,
            _phantom: PhantomData,
        }
    }

    /// create the command from a subcommand reference
    pub fn from(sev: &'a mut impl AsRawFd, subcmd: &'a T) -> Self {
        Self {
            code: T::ID,
            data: subcmd as *const T as _,
            error: 0,
            sev_fd: sev.as_raw_fd() as _,
            _phantom: PhantomData,
        }
    }

    /// encapsulate a `std::io::Error` in an `Indeterminate<Error>`
    pub fn encapsulate(&self, err: std::io::Error) -> Indeterminate<Error> {
        match self.error {
            0 => Indeterminate::<Error>::from(err),
            _ => Indeterminate::<Error>::from(self.error),
        }
    }
}
