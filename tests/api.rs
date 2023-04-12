// SPDX-License-Identifier: Apache-2.0

use snp::firmware::host::{
    CertTableEntry, CertType, Config, ExtConfig, Firmware, PlatformStatus, TcbVersion,
};

use serial_test::serial;

#[cfg_attr(not(has_sev), ignore)]
#[test]
fn get_identifier() {
    let mut fw = Firmware::open().unwrap();
    let id = fw.get_identifier().unwrap();
    assert_ne!(Vec::from(id), vec![0u8; 64]);
}

#[cfg_attr(not(has_sev), ignore)]
#[test]
fn platform_status() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let status: PlatformStatus = fw.platform_status().unwrap();

    println!(
        "Platform status ioctl results:
              version (major, minor): {}.{}
              build id: {}
              guests: {}
              platform tcb microcode version: {}
              platform tcb snp version: {}
              platform tcb tee version: {}
              platform tcb bootloader version: {}
              reported tcb microcode version: {}
              reported tcb snp version: {}
              reported tcb tee version: {}
              reported tcb bootloader version: {}
              state: {}",
        status.version.major,
        status.version.minor,
        status.build_id,
        status.guest_count,
        status.platform_tcb_version.microcode,
        status.platform_tcb_version.snp,
        status.platform_tcb_version.tee,
        status.platform_tcb_version.bootloader,
        status.reported_tcb_version.microcode,
        status.reported_tcb_version.snp,
        status.reported_tcb_version.tee,
        status.reported_tcb_version.bootloader,
        status.state
    );
}

fn build_ext_config(cert: bool, cfg: bool) -> ExtConfig {
    let test_cfg: Config = Config::new(TcbVersion::new(1, 0, 1, 1), 31);

    let cert_table: Vec<CertTableEntry> = vec![
        CertTableEntry::new(CertType::ARK, vec![1; 28]),
        CertTableEntry::new(CertType::ASK, vec![1; 28]),
    ];

    match (cert, cfg) {
        (true, true) => ExtConfig::new(test_cfg, cert_table),
        (true, false) => ExtConfig::new_certs_only(cert_table),
        (false, true) => ExtConfig::new_config_only(test_cfg),
        (false, false) => ExtConfig::default(),
    }
}

#[cfg_attr(not(all(has_sev, feature = "dangerous_hw_tests")), ignore)]
#[ignore]
#[test]
#[serial]
fn snp_set_ext_config_std() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let new_config: ExtConfig = build_ext_config(true, true);
    fw.set_ext_config(new_config).unwrap();
    fw.reset_config().unwrap();
}

#[cfg_attr(not(all(has_sev, feature = "dangerous_hw_tests")), ignore)]
#[ignore]
#[test]
#[serial]
fn snp_set_ext_config_certs_only() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let new_config: ExtConfig = build_ext_config(true, false);
    fw.set_ext_config(new_config).unwrap();
    fw.reset_config().unwrap();
}

#[cfg_attr(not(all(has_sev, feature = "dangerous_hw_tests")), ignore)]
#[ignore]
#[test]
#[serial]
fn snp_set_ext_config_cfg_only() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let new_config: ExtConfig = build_ext_config(false, true);
    fw.set_ext_config(new_config).unwrap();
    fw.reset_config().unwrap();
}

#[cfg_attr(not(all(has_sev, feature = "dangerous_hw_tests")), ignore)]
#[ignore]
#[test]
#[serial]
fn snp_set_ext_invalid_config_std() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let platform_status: PlatformStatus = fw.platform_status().unwrap();

    // Using Current TCB as Committed TCB is not available at the moment,
    // but ideally we would like to check Reported TCB <= Committed TCB, only.
    let mut invalid_tcb: TcbVersion = platform_status.platform_tcb_version;
    invalid_tcb.snp += 1;
    fw.set_ext_config(ExtConfig::new_config_only(Config::new(invalid_tcb, 0)))
        .unwrap();
    fw.reset_config().unwrap();
}

#[cfg_attr(not(all(has_sev, feature = "dangerous_hw_tests")), ignore)]
#[ignore]
#[test]
#[serial]
fn snp_get_ext_config_std() {
    let mut fw: Firmware = Firmware::open().unwrap();
    let hw_config: ExtConfig = fw.get_ext_config().unwrap();
    println!("{:?}", hw_config);
}
