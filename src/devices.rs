use pasedid::config::Config;
use pasedid::datamodel::base_block::DisplayDescriptor;
use pasedid::parser::edid::parse_edid;
use walkdir::WalkDir;

#[derive(Debug)]
struct Device {
    product_name: String,
    product_serial_number: Option<String>,
}

fn list_edid_devices() -> Vec<Device> {
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

        devices.push(Device {
            product_name: product_name.unwrap(),
            product_serial_number,
        });
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_with_edid() {
        let devices = list_edid_devices();
        for device in devices {
            println!("Device: {:?}", device);
        }
    }
}
