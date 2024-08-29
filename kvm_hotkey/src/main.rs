/*
* The Startech SV211HDUA KVM switch seem to be manufactured by Uniclass. Their product code is
* UHI-TA2. Many "manufacturers" have products similar or equal to this device, so the software
* most likely work with others too. YMMV.
*/
use clap::{crate_version, Parser, Subcommand};
use clap_num::{maybe_hex, number_range};
use comfy_table::Table;
use core::panic;
use rusb::{Context, Device, DeviceHandle, Direction, Result, UsbContext};
use std::{time::Duration, u8};

// device uid pid are picked directly form `lsusb` result
const VID: &str = "0x10d5";
const PID: &str = "0x55a2";

/// Filter values between 10 and 60
fn between_10_and_60(s: &str) -> std::result::Result<u8, String> {
    number_range(s, 10, 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_10() {
        let s = "10";
        let result = between_10_and_60(s);
        assert!(result.is_ok());
    }
    #[test]
    fn test_value_60() {
        let s = "60";
        let result = between_10_and_60(s);
        assert!(result.is_ok());
    }
    #[test]
    fn test_value_less_than_10() {
        let s = "6";
        let result = between_10_and_60(s);
        assert!(result.is_err());
    }
    #[test]
    fn test_value_more_than_60() {
        let s = "61";
        let result = between_10_and_60(s);
        assert!(result.is_err());
    }
}

#[derive(Parser)]
#[command(author, version = crate_version!(), about = "Interact with a Startech SV211HDUA KVM switch", long_about = None)]
struct Args {
    #[clap(short, long, help = "vendor id", value_parser=maybe_hex::<u16>, default_value = VID)]
    vendor_id: u16,
    #[clap(short, long, help = "product id", value_parser=maybe_hex::<u16>, default_value = PID)]
    product_id: u16,
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Information about the KVM switch")]
    Info,
    // #[arg(short, long, help = "show information about the KVM switch")]
    #[command(about = "Port to switch to", value_parser = clap::value_parser(u8))]
    Port { port: u8 },
    #[command(about = "Set KVM parameters", value_parser = clap::value_parser(u8), flatten_help = true)]
    Set {
        #[command(subcommand)]
        setcmd: SetCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum SetCommands {
    #[command(about = "fix audio to port", value_parser=clap::value_parser(u8))]
    AudioPort {
        #[clap(help = "Once set audio will not follow display")]
        audio_port: u8,
    },
    #[command(about = "auto scan period")]
    AutoScan {
        #[clap(help="Interval to wait before scanning, in seconds (10-60)", value_parser=between_10_and_60)]
        auto_scan: u8,
    },
}

fn print_info(args: Args, kvminfo: &mut KvmInfo) {
    let mut table = Table::new();
    table
        .set_header(vec!["Description", "Value"])
        .add_row(vec!["Bus", format!("{}", kvminfo.bus).as_str()])
        .add_row(vec!["Device", format!("{}", kvminfo.device).as_str()])
        .add_row(vec!["Manufacturer", kvminfo.manufacturer.as_str()])
        .add_row(vec!["Product", kvminfo.product.as_str()])
        .add_row(vec![
            "Vendor ID",
            (format!("0x{:04x}", args.vendor_id)).as_str(),
        ])
        .add_row(vec![
            "Product ID",
            format!("0x{:04x}", args.product_id).as_str(),
        ])
        .add_row(vec![
            "Connected computers",
            format!("{}", kvminfo.connected_comps).as_str(),
        ])
        .add_row(vec![
            "Connected port",
            format!("{}", kvminfo.port_num).as_str(),
        ]);

    println!("Information about the switch:\n\n{table}");
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut context = Context::new()?;

    let (mut device, mut handle) = open_device(&mut context, args.vendor_id, args.product_id)
        .unwrap_or_else(|| {
            panic!(
                "Failed to open USB device. VID: 0x{:4x} PID: 0x{:04x}",
                args.vendor_id, args.product_id
            )
        });

    let mut kvminfo = KvmInfo {
        bus: device.bus_number(),
        device: device.address(),
        ..Default::default()
    };
    let _ = match get_kvm_info(&mut handle, &mut kvminfo) {
        Ok(_) => true,
        Err(e) => {
            panic!("Failed to query KVM:\n{}", e)
        }
    };

    let endpoints = find_readable_endpoints(&mut device)?;
    let (endpoint, _direction) = endpoints
        .iter()
        .find(|(_, y)| *y == Direction::Out)
        .expect("No Configurable endpoint found on device");

    let _has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface)?;
            true
        }
        _ => false,
    };

    match args.cmd {
        Commands::Port { port } => {
            println!("Switching to port {port}...");
            println!("Sending magic bytes");
            send_magic(&mut handle, endpoint.address, &port)?;
        }
        Commands::Info => {
            print_info(args, &mut kvminfo);
        }
        Commands::Set { setcmd } => match setcmd {
            SetCommands::AutoScan { auto_scan } => {
                println!("Setting auto scan period to {auto_scan}")
            }
            SetCommands::AudioPort { audio_port } => {
                println!("Fixing audio port to {audio_port}")
            }
        },
    }

    // cleanup after use
    // handle.release_interface(endpoint.iface)?;
    // handle.attach_kernel_driver(endpoint.iface)?;

    Ok(())
}

/*
* To switch from port 1 to port 2:
* 0000   01 01 00 00 00 00 00 00
*
* To switch from port 2 to port 1:
* 0000   01 00 00 00 00 00 00 00
*/
fn send_magic<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    address: u8,
    port_num: &u8,
) -> Result<usize> {
    let data: [u8; 8] = [0x01, *port_num - 1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    println!("{:?}", data);
    let timeout = Duration::from_millis(100);

    handle.write_interrupt(address, &data, timeout)
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

/*
* Serial number to reflect what's connected
*
* Connected to port 1, only one computer connected:
* 0000   30 00 12 00 92 00
*              ^     ^
*              |     - 1001 0010
*              |
*              - 0001 0010
*
* Connected to port 1, two computers connected:
* 0000   30 00 32 00 92 00
*              ^     ^
*              |     - 1001 0010
*              |
*              - 0011 0010
*
* Connected to port 2, only one computer connected:
* 0000   31 00 22 00 b2 00
*              ^     ^
*              |     - 1011 0010
*              |
*              - 0010 0010
*
* Connected to port 2, two computers connected:
* 0000   31 00 32 00 b2 00
*              ^     ^
*              |     - 1011 0010
*              |
*              - 0010 0010
*
* The only bit that could tell us which one it's connected to is the 4th one (0 indexed, 92/b2)
* We AND that with 0b1001_0000 and bitshift 4 to get the port, e.g. 0x92 AND 0x60 >> 4
*/
#[derive(Debug, Default)]
struct KvmInfo {
    connected_comps: u8,
    port_num: u8,
    product: String,
    manufacturer: String,
    bus: u8,
    device: u8,
}

fn get_kvm_info<T: UsbContext>(handle: &mut DeviceHandle<T>, kvminfo: &mut KvmInfo) -> Result<()> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    if !languages.is_empty() {
        let language = languages[0];

        let serial_num_str = handle
            .read_serial_number_string(language, &device_desc, timeout)
            .unwrap();

        let byte_vec = serial_num_str.as_bytes();
        // println!("{:x?}", byte_vec);
        let connected_comps = ((byte_vec[1] >> 5) & 0b0000_0011) + 1; // 0 based so add 1
        let port_num = ((byte_vec[3] >> 5) & 0b0000_0011) + 1; // 0 based so add 1
        kvminfo.connected_comps = connected_comps;
        kvminfo.port_num = port_num;
        kvminfo.product = handle
            .read_product_string(language, &device_desc, timeout)
            .unwrap();
        kvminfo.manufacturer = handle
            .read_manufacturer_string(language, &device_desc, timeout)
            .unwrap();
    }
    Ok(())
}

#[derive(Debug)]
struct Endpoint {
    // config: u8,
    iface: u8,
    // setting: u8,
    address: u8,
}

// returns all readable endpoints for given usb device and descriptor
fn find_readable_endpoints<T: UsbContext>(
    device: &mut Device<T>,
) -> Result<Vec<(Endpoint, Direction)>> {
    let device_desc = device.device_descriptor()?;
    let mut endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // println!("{:#?}", config_desc);
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                // println!("{:#?}", interface_desc);
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    // println!("{:#?}", endpoint_desc);
                    endpoints.push((
                        Endpoint {
                            // config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            // setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        },
                        endpoint_desc.direction() as Direction,
                    ));
                }
            }
        }
    }

    Ok(endpoints)
}
