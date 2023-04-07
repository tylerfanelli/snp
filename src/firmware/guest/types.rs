// SPDX-License-Identifier: Apache-2.0
use bitfield::bitfield;

// Export types from the FFI layer which are applicable to the Rust-friendly APIs.
pub use crate::firmware::linux::guest::types::{
    AttestationReport, GuestPolicy, PlatformInfo, TcbVersion,
};

pub(crate) use crate::firmware::linux::guest as GuestFFI;
pub(crate) use crate::firmware::linux::host as HostFFI;

#[derive(Copy, Clone, Debug)]
/// Structure of required data for fetching the derived key.
pub struct DerivedKey {
    /// Selects the root key to derive the key from.
    /// 0: Indicates VCEK.
    /// 1: Indicates VMRK.
    root_key_select: u32,

    /// Reserved, must be zero
    _reserved_0: u32,

    /// What data will be mixed into the derived key.
    pub guest_field_select: GuestFieldSelect,

    /// The VMPL to mix into the derived key. Must be greater than or equal
    /// to the current VMPL.
    pub vmpl: u32,

    /// The guest SVN to mix into the key. Must not exceed the guest SVN
    /// provided at launch in the ID block.
    pub guest_svn: u32,

    /// The TCB version to mix into the derived key. Must not
    /// exceed CommittedTcb.
    pub tcb_version: u64,
}

impl DerivedKey {
    /// Create a new instance for requesting an DerivedKey.
    pub fn new(
        root_key_select: bool,
        guest_field_select: GuestFieldSelect,
        vmpl: u32,
        guest_svn: u32,
        tcb_version: u64,
    ) -> Self {
        Self {
            root_key_select: u32::from(root_key_select),
            _reserved_0: Default::default(),
            guest_field_select,
            vmpl,
            guest_svn,
            tcb_version,
        }
    }

    /// Obtain a copy of the root key select value (Private Field)
    pub fn get_root_key_select(&self) -> u32 {
        self.root_key_select
    }
}

bitfield! {
    /// Data which will be mixed into the derived key.
    ///
    /// | Bit(s) | Name | Description |
    /// |--------|------|-------------|
    /// |0|GUEST_POLICY|Indicates that the guest policy will be mixed into the key.|
    /// |1|IMAGE_ID|Indicates that the image ID of the guest will be mixed into the key.|
    /// |2|FAMILY_ID|Indicates the family ID of the guest will be mixed into the key.|
    /// |3|MEASUREMENT|Indicates the measurement of the guest during launch will be mixed into the key.|
    /// |4|GUEST_SVN|Indicates that the guest-provided SVN will be mixed into the key.|
    /// |5|TCB_VERSION|Indicates that the guest-provided TCB_VERSION will be mixed into the key.|
    /// |63:6|\-|Reserved. Must be zero.|
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    pub struct GuestFieldSelect(u64);
    impl Debug;
    /// Check/Set guest policy inclusion in derived key.
    pub get_guest_policy, set_guest_policy: 0, 0;
    /// Check/Set image id inclusion in derived key.
    pub get_image_id, set_image_id: 1, 1;
    /// Check/Set family id inclusion in derived key.
    pub get_family_id, set_family_id: 2, 2;
    /// Check/Set measurement inclusion in derived key.
    pub get_measurement, set_measurement: 3, 3;
    /// Check/Set svn inclusion in derived key.
    pub get_svn, set_svn: 4, 4;
    /// Check/Set tcb version inclusion in derived key.
    pub get_tcb_version, set_tcb_version: 5, 5;
}
