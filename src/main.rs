mod usx_structs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_yaml_file>", args[0]);
        std::process::exit(1);
    }
    let yaml_file_path = &args[1];
    let yaml_content = std::fs::read_to_string(yaml_file_path)?;
    let mut usx_config: usx_structs::USXConfig = serde_yaml::from_str(&yaml_content)?;

    let just_list_devices = usx_config.list_only.unwrap_or(false);
    let mut results: std::collections::HashMap<String, usx_structs::USXDevice> =
        std::collections::HashMap::new();

    let mut usb_devices: Vec<usx_structs::UsbPortInfoWithPort> = vec![];

    let ports = serialport::available_ports()?;
    println!("Discovered USB devices:");
    for port in &ports {
        if let serialport::SerialPortType::UsbPort(si) = &port.port_type {
            if let Some(skip_ports) = &usx_config.skip_ports {
                if skip_ports.contains(&port.port_name) {
                    continue;
                }
            }
            let discovered_si = usx_structs::UsbPortInfoWithPort {
                port: port.port_name.clone(),
                port_info: si.clone(),
            };
            println!("{}", discovered_si);
            if !just_list_devices {
                usb_devices.push(discovered_si.clone());
            }
        }
    }
    println!("***********");

    if let Some(mut devices) = usx_config.devices {
        while let Some(mut device) = devices.pop() {
            println!("Looking for device:\n{}", device);
            for usb_device in &usb_devices {
                if let Some(skip_ports) = &usx_config.skip_ports {
                    if skip_ports.contains(&usb_device.port) {
                        continue;
                    }
                }
                println!("Testing USB port: {}", &usb_device.port);
                println!("##########");
                // Before checking the identification command, check the rest of the fields
                if !device.none_port_info() && device != *usb_device {
                    continue;
                }
                // Skip devices that do not have a serial identification command
                // but add them if their port info match
                if !device.can_identify_through_serial() {
                    if device == *usb_device {
                        results.insert(usb_device.port.clone(), device.clone());
                    }
                    continue;
                }
                if device.test_port(
                    usb_device.port.clone(),
                    usx_config.serial_connection_timeout_ms,
                ) {
                    match &mut usx_config.skip_ports {
                        Some(skip_ports) => {
                            skip_ports.push(usb_device.port.clone());
                        }
                        None => {
                            usx_config.skip_ports = Some(vec![usb_device.port.clone()]);
                        }
                    }
                    results.insert(usb_device.port.clone(), device.clone());
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
    }
    println!("-------------");
    println!("Discovered Results:");
    println!("-------------");
    for (port, device) in &results {
        println!("Port:\n{}\nDevice:\n{}", port, device);
    }
    Ok(())
}
