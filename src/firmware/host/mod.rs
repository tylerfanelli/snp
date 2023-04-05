// SPDX-License-Identifier: Apache-2.0

//! # Host / Platform Owner
//!
//! The Platform Owner, Host, or Cloud Service Provider (CSP). This is the system software, including the hypervisor, where a confidential virtual-machine (VM) or container will be deployed.

pub mod types;

use std::{
    fs::{File, OpenOptions},
    os::fd::{AsRawFd, RawFd},
};

use crate::error::*;
use types::*;

use FFI::ioctl::*;

use self::types::FFI::types::GetId;

///
/// This is a façade function to give public access to the FFI parse table
/// function.
///
pub fn parse_table(data: RawData) -> Result<Vec<CertTableEntry>, uuid::Error> {
    match data {
        RawData::Pointer(pointer) => unsafe {
            FFI::types::CertTableEntry::parse_table(pointer as *mut FFI::types::CertTableEntry)
        },
        RawData::Vector(vector) => unsafe {
            FFI::types::CertTableEntry::parse_table(
                vector.as_ptr() as *mut FFI::types::CertTableEntry
            )
        },
    }
}

/// A handle to the SEV platform.
pub struct Firmware(File);

impl Firmware {
    /// Create a handle to the SEV platform via `/dev/sev`.
    ///
    /// # Example:
    /// ```ignore
    /// let mut firmware: Firmware = Firmware::open().unwrap();
    /// ```
    pub fn open() -> std::io::Result<Firmware> {
        Ok(Firmware(
            OpenOptions::new().read(true).write(true).open("/dev/sev")?,
        ))
    }

    /// Get the unique CPU identifier.
    ///
    /// This is especially helpful for sending AMD an HTTP request to fetch
    /// the signed CEK certificate.
    pub fn get_identifier(&mut self) -> Result<Identifier, Indeterminate<Error>> {
        let mut bytes = [0u8; 64];
        let mut id = GetId::new(&mut bytes);

        GET_ID.ioctl(&mut self.0, &mut Command::from_mut(&mut id))?;

        Ok(Identifier(id.as_slice().to_vec()))
    }

    /// Query the SNP platform status.
    ///
    /// # Example:
    /// ```ignore
    /// use snp::firmware::host::*;
    ///
    /// let mut firmware: Firmware = Firmware::open().unwrap();
    ///
    /// let status: SnpPlatformStatus = firmware.snp_platform_status().unwrap();
    /// ```
    pub fn snp_platform_status(&mut self) -> Result<SnpPlatformStatus, Indeterminate<Error>> {
        let mut platform_status: SnpPlatformStatus = SnpPlatformStatus::default();
        SNP_PLATFORM_STATUS.ioctl(&mut self.0, &mut Command::from_mut(&mut platform_status))?;
        Ok(platform_status)
    }

    /// Reset the configuration of the AMD secure processor. Useful for resetting the committed_tcb.
    /// # Example:
    /// ```ignore
    /// use snp::firmware::host::*;
    ///
    /// let mut firmware: Firmware = Firmware::open().unwrap();
    ///
    /// let reset: bool = firmware.snp_reset_config().unwrap();
    /// ```
    pub fn snp_reset_config(&mut self) -> Result<bool, UserApiError> {
        let mut config: FFI::types::SnpSetExtConfig = FFI::types::SnpSetExtConfig {
            config_address: 0,
            certs_address: 0,
            certs_len: 0,
        };

        SNP_SET_EXT_CONFIG.ioctl(&mut self.0, &mut Command::from_mut(&mut config))?;
        Ok(true)
    }
    /// Fetch the SNP Extended Configuration.
    ///
    /// # Example:
    /// ```ignore
    /// let mut firmware: Firmware = Firmware::open().unwrap();
    ///
    /// let status: SnpExtConfig = firmware.snp_get_ext_config().unwrap();
    /// ```
    pub fn snp_get_ext_config(&mut self) -> Result<SnpExtConfig, UserApiError> {
        let mut raw_buf: Vec<u8> = vec![0; _4K_PAGE];
        let mut config: FFI::types::SnpGetExtConfig = FFI::types::SnpGetExtConfig {
            config_address: 0,
            certs_address: raw_buf.as_mut_ptr() as *mut CertTableEntry as u64,
            certs_len: _4K_PAGE as u32,
        };

        SNP_GET_EXT_CONFIG
            .ioctl(&mut self.0, &mut Command::from_mut(&mut config))
            .or_else(|err| {
                // If the error occurred because the buffer was to small, it will have changed
                // the buffer. If it has, we will attempt to resize it.
                if config.certs_len <= _4K_PAGE as u32 {
                    return Err(err);
                }

                raw_buf = vec![0; config.certs_len as usize];
                config.certs_address = raw_buf.as_ptr() as *const CertTableEntry as u64;
                SNP_GET_EXT_CONFIG.ioctl(&mut self.0, &mut Command::from_mut(&mut config))
            })?;

        config.try_into().map_err(|op: uuid::Error| op.into())
    }

    /// Set the SNP Extended Configuration.
    ///
    /// # Example:
    /// ```ignore
    /// pub const ARK: &[u8] = include_bytes!("../../certs/builtin/milan/ark.pem");
    /// pub const ASK: &[u8] = include_bytes!("../../certs/builtin/genoa/ask.pem");
    /// pub const VCEK: &[u8] = include_bytes!("vcek.pem");
    ///
    /// let configuration: SnpConfig = SnpConfig::new(
    ///     TcbVersion::new(3, 0, 10, 169),
    ///     0,
    /// );
    ///
    /// // Generate a vector of certificates to store in hypervisor memory.
    /// let certificates: Vec<CertTableEntry> = vec![
    ///     CertTableEntry::new(SnpCertType::ARK, ARK.to_vec()),
    ///     CertTableEntry::new(SnpCertType::ASK, ASK.to_vec()),
    ///     CertTableEntry::new(SnpCertType::VCEK, VCEK.to_vec()),
    /// ];
    ///
    /// // Call the `new` constructor to generate the extended configuration.
    /// let ext_config: SnpExtConfig = SnpExtConfig::new(configuration, certificates);
    ///
    /// let mut firmware: Firmware = Firmware::open().unwrap();
    ///
    /// let status: bool = firmware.snp_set_ext_config(ext_config).unwrap();
    /// ```
    pub fn snp_set_ext_config(
        &mut self,
        mut new_config: SnpExtConfig,
    ) -> Result<bool, UserApiError> {
        let mut bytes: Vec<u8> = vec![];

        if let Some(ref mut certificates) = new_config.certs {
            bytes = FFI::types::CertTableEntry::uapi_to_vec_bytes(certificates)?;
        }

        let mut new_ext_config: FFI::types::SnpSetExtConfig = new_config.try_into()?;
        new_ext_config.certs_address = bytes.as_mut_ptr() as u64;

        SNP_SET_EXT_CONFIG.ioctl(&mut self.0, &mut Command::from_mut(&mut new_ext_config))?;
        Ok(true)
    }
}

impl AsRawFd for Firmware {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
