#[derive(Clone, Debug)]
pub struct UsbPortInfoWithPort {
    pub port: String,
    pub port_info: serialport::UsbPortInfo,
}

impl std::fmt::Display for UsbPortInfoWithPort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "-------------")?;
        writeln!(f, "USB Port: {}", self.port)?;
        writeln!(f, "Vendor ID: {}", self.port_info.vid)?;
        writeln!(f, "Product ID: {}", self.port_info.pid)?;
        if let Some(manufacturer) = &self.port_info.manufacturer {
            writeln!(f, "Manufacturer: {}", manufacturer)?;
        }
        if let Some(product) = &self.port_info.product {
            writeln!(f, "Product: {}", product)?;
        }
        if let Some(serial_number) = &self.port_info.serial_number {
            writeln!(f, "Serial Number: {}", serial_number)?;
        }
        writeln!(f, "-------------")?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct USXDevice {
    pub baudrate: Option<u32>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub serial_identification_command: Option<String>,
    pub serial_identification_command_expected_response: Option<String>,
    pub serial_commands_after_identification: Option<Vec<String>>,
    pub system_commands_after_identification: Option<Vec<String>>,
    #[serde(skip)]
    serial_commands_output: Option<String>,
    #[serde(skip)]
    system_commands_output: Option<String>,
}

impl USXDevice {
    fn escape_string(&self, input: &str) -> String {
        let mut escaped = String::new();
        for c in input.chars() {
            match c {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                '\'' => escaped.push_str("\\'"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                _ => escaped.push(c),
            }
        }
        escaped
    }

    pub fn none_port_info(&self) -> bool {
        return self.manufacturer.is_none()
            && self.product.is_none()
            && self.serial_number.is_none()
            && self.vendor_id.is_none()
            && self.product_id.is_none();
    }

    pub fn can_identify_through_serial(&self) -> bool {
        return self.serial_identification_command.is_some()
            && self
                .serial_identification_command_expected_response
                .is_some()
            && self.baudrate.is_some();
    }

    fn read_serial(&self, s_p: &mut Box<dyn serialport::SerialPort>) -> String {
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            match s_p.read(&mut byte) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    if byte[0] == b'\n' {
                        break;
                    }
                    buffer.push(byte[0]);
                }
                Err(e) => {
                    eprintln!("Error in read_serial: {:?}", e);
                    break;
                }
            }
        }
        return String::from_utf8(buffer)
            .unwrap_or(String::new())
            .trim()
            .to_string();
    }

    pub fn test_port(&mut self, port: String, timeout: u64) -> bool {
        let sp = serialport::new(&port, self.baudrate.unwrap())
            .timeout(std::time::Duration::from_millis(timeout))
            .open();
        match sp {
            Ok(mut s_p) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match s_p.write_all(
                    self.serial_identification_command
                        .clone()
                        .unwrap()
                        .as_bytes(),
                ) {
                    Ok(_) => {
                        let serial_response = self.read_serial(&mut s_p);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if serial_response
                            == self
                                .serial_identification_command_expected_response
                                .clone()
                                .unwrap()
                        {
                            if self.system_commands_after_identification.is_some() {
                                for command_string in
                                    self.system_commands_after_identification.clone().unwrap()
                                {
                                    let mut parts = command_string.split_whitespace();
                                    let command = parts.next().unwrap_or("");
                                    let args: Vec<&str> = parts.collect();

                                    let output =
                                        std::process::Command::new(command).args(&args).output();

                                    if let Ok(out) = output {
                                        let stdout =
                                            String::from_utf8_lossy(&out.stdout).to_string();
                                        let stderr =
                                            String::from_utf8_lossy(&out.stderr).to_string();
                                        self.system_commands_output = Some(
                                            self.system_commands_output
                                                .clone()
                                                .unwrap_or(String::new())
                                                + &stdout
                                                + &stderr
                                                + "\n---\n",
                                        );
                                    }
                                }
                            }
                            if self.serial_commands_after_identification.is_some() {
                                for serial_command in
                                    self.serial_commands_after_identification.clone().unwrap()
                                {
                                    if let Err(e) = s_p.write_all(serial_command.as_bytes()) {
                                        println!(
                                            "Error {} while writing {} on {}",
                                            e, serial_command, &port
                                        );
                                    }
                                    self.serial_commands_output = Some(
                                        self.serial_commands_output
                                            .clone()
                                            .unwrap_or(String::new())
                                            + &self.read_serial(&mut s_p)
                                            + "\n---\n",
                                    );
                                }
                            }
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        eprintln!("1 {}", e);
                        return false;
                    }
                }
            }
            Err(e) => {
                eprintln!("2 {}", e);
                return false;
            }
        }
    }
}

impl PartialEq<UsbPortInfoWithPort> for USXDevice {
    fn eq(&self, other: &UsbPortInfoWithPort) -> bool {
        return (self.manufacturer.is_none() || self.manufacturer == other.port_info.manufacturer)
            && (self.product.is_none() || self.product == other.port_info.product)
            && (self.serial_number.is_none()
                || self.serial_number == other.port_info.serial_number)
            && (self.vendor_id.is_none() || self.vendor_id.unwrap() == other.port_info.vid)
            && (self.product_id.is_none() || self.product_id.unwrap() == other.port_info.pid);
    }
}

impl std::fmt::Display for USXDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "-------------")?;
        if let Some(baudrate) = &self.baudrate {
            writeln!(f, "Baudrate: {}", baudrate)?;
        }
        if let Some(vid) = &self.vendor_id {
            writeln!(f, "Vendor ID: {}", vid)?;
        }
        if let Some(pid) = &self.product_id {
            writeln!(f, "Product ID: {}", pid)?;
        }
        if let Some(manufacturer) = &self.manufacturer {
            writeln!(f, "Manufacturer: {}", manufacturer)?;
        }
        if let Some(product) = &self.product {
            writeln!(f, "Product: {}", product)?;
        }
        if let Some(serial_number) = &self.serial_number {
            writeln!(f, "Serial Number: {}", serial_number)?;
        }
        if let Some(serial_identification_command) = &self.serial_identification_command {
            writeln!(
                f,
                "Serial identification command: {}",
                &self.escape_string(serial_identification_command)
            )?;
        }
        if let Some(serial_identification_command_expected_response) =
            &self.serial_identification_command_expected_response
        {
            writeln!(
                f,
                "Serial identification command expected response: {}",
                &self.escape_string(serial_identification_command_expected_response)
            )?;
        }
        if let Some(serial_commands_after_identification) =
            &self.serial_commands_after_identification
        {
            writeln!(f, "Serial commands after identification:")?;
            for serial_command in serial_commands_after_identification {
                writeln!(f, "{}", &self.escape_string(serial_command))?;
            }
        }
        if let Some(system_commands_after_identification) =
            &self.system_commands_after_identification
        {
            writeln!(f, "System commands after identification:")?;
            for system_command in system_commands_after_identification {
                writeln!(f, "{}", &self.escape_string(system_command))?;
            }
        }
        if let Some(serial_command_output) = &self.serial_commands_output {
            writeln!(f, "Serial command output:\n{}", serial_command_output)?;
        }
        if let Some(system_command_output) = &self.system_commands_output {
            writeln!(f, "System command output:\n{}", system_command_output)?;
        }
        writeln!(f, "-------------")?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct USXConfig {
    pub serial_connection_timeout_ms: u64,
    pub list_only: Option<bool>,
    pub skip_ports: Option<Vec<String>>,
    pub devices: Option<Vec<USXDevice>>,
}
