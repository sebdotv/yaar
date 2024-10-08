use itertools::Itertools;
use pasedid::config::Config;
use pasedid::datamodel::base_block::DisplayDescriptor;
use pasedid::parser::edid::parse_edid;
use std::path::{Component, Path};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub product_name: String,
    pub product_serial_number: Option<String>,
    pub edid_bytes: Vec<u8>,
    pub edid_hex: String,
}

pub fn list_edid_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    for entry in WalkDir::new("/sys/devices/") {
        let entry = entry.unwrap();
        if entry.file_name() != "edid" {
            continue;
        }

        let bytes = std::fs::read(entry.path()).unwrap();
        if bytes.is_empty() {
            continue;
        }

        let name = get_device_name(entry.path());

        let (remaining, edid) = parse_edid(
            &bytes,
            Config {
                handle_extensions: true,
            },
        )
        .unwrap();
        assert!(remaining.is_empty());

        let mut product_name: Option<String> = None;
        let mut product_serial_number: Option<String> = None;

        for descriptor in edid.base_block.display_descriptors.0 {
            match descriptor {
                DisplayDescriptor::ProductName(pn) => {
                    assert!(product_name.is_none());
                    product_name = Some(pn.name);
                }
                DisplayDescriptor::ProductSerialNumber(psn) => {
                    assert!(product_serial_number.is_none());
                    product_serial_number = Some(psn.serial_number);
                }
                _ => {}
            }
        }

        assert!(product_name.is_some());
        let product_name = product_name.unwrap();

        devices.push(Device {
            name,
            product_name,
            product_serial_number,
            edid_hex: hex::encode(&bytes),
            edid_bytes: bytes,
        });
    }

    devices
}

fn get_device_name(path: &Path) -> String {
    let (c, b, a) = path
        .components()
        .rev()
        .skip(1)
        .take(3)
        .map(|c| match c {
            Component::Normal(s) => s.to_str().unwrap(),
            _ => panic!("unexpected component"),
        })
        .collect_tuple()
        .unwrap();
    assert_eq!(a, "drm");
    let name = c
        .strip_prefix(&format!("{}-", b))
        .expect("unexpected prefix");
    name.to_string()
}
