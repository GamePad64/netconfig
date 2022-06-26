#[cfg(any(target_os = "macos", target_os = "windows"))]
use netconfig::sys::MetadataExt;
use netconfig::{list_addresses, list_interfaces, InterfaceHandle};

use clap::{Parser, Subcommand};
use netconfig::sys::InterfaceHandleExt;

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
            for handle in list_interfaces().iter() {
                let metadata = handle.metadata().unwrap();
                println!("Index: {}", metadata.index());
                println!("Name: {}", metadata.name());
                cfg_if::cfg_if! {
                    if #[cfg(any(target_os = "macos", target_os = "windows"))] {
                        println!("Alias: {}", metadata.alias());
                    }
                }
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "windows")] {
                        println!("GUID: {:?}", metadata.guid());
                        println!("LUID: {:?}", metadata.luid());
                    }
                }
                println!("MTU: {}", metadata.mtu());

                for address in handle.get_addresses().unwrap() {
                    println!("Address: {:?}", address);
                }
                println!();
            }
        }
        #[cfg(unix)]
        Commands::SetIfParam { iface, param } => {
            let handle = InterfaceHandle::try_from_name(&*iface).unwrap();
            match param {
                IfParam::Up => handle.set_up(true).unwrap(),
                IfParam::Down => handle.set_up(false).unwrap(),
            }
        }
    }
}
