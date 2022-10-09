use netconfig::{list_addresses, list_interfaces, Interface};

use clap::{Parser, Subcommand};
use netconfig::sys::InterfaceExt;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ListInterfaces,
    ListAddresses,
    #[cfg(unix)]
    SetIfParam {
        iface: String,
        #[clap(subcommand)]
        param: IfParam,
    },
}

#[derive(Debug, Subcommand)]
enum IfParam {
    Up,
    Down,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::ListAddresses => {
            println!("Addresses: {:?}", list_addresses())
        }
        Commands::ListInterfaces => {
            for handle in list_interfaces().unwrap().iter() {
                println!("Index: {}", handle.index().unwrap());
                println!("Name: {}", handle.name().unwrap());
                cfg_if::cfg_if! {
                    if #[cfg(any(target_os = "macos", target_os = "windows"))] {
                        println!("Alias: {}", handle.alias().unwrap());
                    }
                }
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "windows")] {
                        println!("GUID: {:?}", handle.guid());
                        println!("LUID: {:?}", handle.luid());
                    }
                }
                println!("MTU: {}", handle.mtu().unwrap());

                for address in handle.addresses().unwrap() {
                    println!("Address: {:?}", address);
                }

                if let Ok(hwaddress) = handle.hwaddress() {
                    println!("MAC: {}", hwaddress.map(|a| hex::encode([a])).join(":"));
                }

                println!();
            }
        }
        #[cfg(unix)]
        Commands::SetIfParam { iface, param } => {
            let handle = Interface::try_from_name(&iface).unwrap();
            match param {
                IfParam::Up => handle.set_up(true).unwrap(),
                IfParam::Down => handle.set_up(false).unwrap(),
            }
        }
    }
}
