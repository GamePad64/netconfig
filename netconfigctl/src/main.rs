#[cfg(any(target_os = "macos", target_os = "windows"))]
use netconfig::sys::MetadataExt;
use netconfig::{list_addresses, list_interfaces};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ListInterfaces,
    ListAddresses,
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
                        println!("LUID: {:?}", unsafe { metadata.luid().Value });
                    }
                }
                println!("MTU: {}", metadata.mtu());

                for address in handle.get_addresses().unwrap() {
                    println!("Address: {:?}", address);
                }
                println!();
            }
        }
    }
}
