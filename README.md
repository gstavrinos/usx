# usx

USB Serial eXplorer: Discover, identify and execute commands for your USB devices

## Purpose

A complex system of USB devices often requires identification of each device before executing some commands.

`usx` tries to semi-automate the process of identifying USB devices and optionally execute serial commands on them and/or system commands.

## Configuration

The available parameters are the following:

- `list_only` (bool): When `true`, `usx` will only list the discovered USB devices.
- `serial_connection_timeout_ms` (uint): When trying to identify a device, this is the time in milliseconds after which `usx` will timeout its serial connection and move on the the next discovered USB device.
- `skip_ports` (array of strings): A collection of ports that `usx` should skip while identifying. Useful when some devices are busy or already known.
- `devices` (array of the following params): This is the part where each device is described. Devices can be described using the following parameters:
  - `manufacturer` (string): The name of the manufacturer, the way it is reported in the device's USB port info
  - `product` (string): The name of the product name, the way it is reported in the device's USB port info
  - `vendor_id` (uint): The vendor ID, the way it is reported in the device's USB port info
  - `product_id` (uint): The product ID, the way it is reported in the device's USB port info
  - `serial_number` (string): The serial number, the way it is reported in the device's USB port info
  - `serial_identification_command` (string): The command to send to the device in order to trigger its identification process. A typical use is the SCPI `*IDN?` command
  - `serial_identification_command_expected_response` (string): The response that needs to be obtained through serial in order to deem successful the identification process
  - baudrate (uint): The baudrate to be used to communicate with the device through serial
  - `serial_commands_after_identification` (array of strings): The serial commands to be sent to the device after it has been identified. They are sent in the specified order
  - `system_commands_after_identification` (array of strings): The system commands to be executed on the host machine after the device has been identified. They are executed in the specified order

An example configuration file is given in the `config` folder of the repository.

### Notes

- Most of the parameters under `devices` are optional, but if a `serial_identification_command` is specified, the `serial_identification_command_expected_response` and `baudrate` must also be specified.

- When specifying `devices` parameters, in order for a device to be considered identified, ALL parameters must much. This means that a device could much when given just a `vendor_id` and `product_id` combination, but not when also give a `serial_number` too. The same applies when also specifying the set of parameters for identification through serial commands.

## How to use it

`cargo run -- <path/to/config_file.yaml>`

e.g.

`cargo run -- config/usx_conf.yaml`

## Typical output

The output of `usx` is divided into 3 parts:

1. Discovered USB devices
2. Identification process for each device specified in the `devices` parameter
3. The identified devices along with output from potential serial and/or system commands.
