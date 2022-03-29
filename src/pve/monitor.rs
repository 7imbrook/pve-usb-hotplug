use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std;
use std::fmt::Debug;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::Path;

use super::commands;

#[derive(Debug)]
pub struct QMPMonitor {
    pub vmid: i32,
    stream: UnixStream,
}

impl QMPMonitor {
    pub fn new(id: i32) -> Result<Self, &'static str> {
        let socket =
            Path::new("/var/run/qemu-server/").join(format!("{}.qmp", id));

        let stream = UnixStream::connect(&socket).unwrap();
        let qmp = QMPMonitor {
            vmid: id,
            stream: stream,
        };
        qmp.init();
        Ok(qmp)
    }

    /**
     * Parse until newline from the qmp socket into the serde type specified
     */
    fn read_message<T>(&self) -> Result<T, &'static str>
    where
        T: DeserializeOwned,
    {
        let mut reader = BufReader::new(&self.stream);
        let mut buf = String::default();
        reader.read_line(&mut buf).unwrap();
        debug!("Recieve Raw: {}", buf);
        let res = serde_json::from_str(&buf);
        if res.is_ok() {
            return Ok(res.unwrap());
        }
        return Err("Failed to decode");
    }

    /**
     * Serializes message as json and sends to qmp socket
     */
    fn send_message<T>(&self, message: &T)
    where
        T: Serialize + Debug,
    {
        debug!("Sending: {:?}", message);
        let msg = serde_json::to_string(message).unwrap();
        let mut writer = BufWriter::new(&self.stream);
        writer.write_all(&msg.as_bytes()).unwrap();
    }

    /**
     * Send and read back a message
     */
    fn execute<T>(
        &self,
        command: &commands::QMPMessage,
    ) -> Result<T, &'static str>
    where
        T: DeserializeOwned,
    {
        self.send_message(command);
        self.read_message()
    }

    fn execute_command<T>(
        &self,
        argument: commands::Argument,
    ) -> Result<T, &'static str>
    where
        T: DeserializeOwned,
    {
        self.execute(&commands::build_command(argument))
    }

    /**
     * QMP requires that the capabilities are negotiated on socket open,
     * we can run this multiple times, but to save on trips just mutate self
     * and track init.
     */
    fn init(&self) {
        // Needs to clear the hello message before sending the first command
        self.read_message::<serde_json::Value>().unwrap();
        debug!("Sending handshake");
        self.execute::<serde_json::Value>(&commands::build_command(
            commands::Argument::Handshake {},
        ))
        .unwrap();
    }

    fn add_xhci_device(&self) {
        info!("Initalizing xhci device");
        if self.get_bool("/machine/peripheral/xhci", "realized") {
            info!("xhci already available");
            return;
        }
        self.execute::<serde_json::Value>(&commands::build_command(
            commands::Argument::DeviceAdd {
                id: "xhci",
                driver: "nec-usb-xhci",
                bus: "pci.1",
                addr: Some("0x1b"),
                vendorid: None,
                productid: None,
            },
        ))
        .unwrap();
    }

    fn get_bool(&self, path: &str, property: &str) -> bool {
        let response = self.execute_command::<commands::response::Bool>(
            commands::Argument::QomGet { path, property },
        );
        response.and_then(|v| Ok(v.value)).unwrap_or(false)
    }
    fn get_string(&self, path: &str, property: &str) -> String {
        let response = self.execute_command::<commands::response::StringVal>(
            commands::Argument::QomGet { path, property },
        );
        response
            .and_then(|v| Ok(v.value))
            .unwrap_or(String::from("unknown"))
    }

    #[allow(dead_code)]
    pub fn list(&self, path: &str) -> Option<serde_json::Value> {
        println!("QOM_GET {}", path);
        self.execute_command::<commands::response::QomList>(
            commands::Argument::QomList { path: path },
        )
        .and_then(|r| {
            r.items.iter().for_each(|item| {
                let k: &str = &item.kind;
                if k.starts_with("child<") || k.starts_with("link<") {
                    println!("{: >25}: {}", item.name, item.kind)
                } else {
                    match &item.kind[..] {
                        "bool" => println!(
                            "{: >25}: {}",
                            item.name,
                            self.get_bool(path, &item.name)
                        ),
                        "string" => println!(
                            "{: >25}: {}",
                            item.name,
                            self.get_string(path, &item.name)
                        ),
                        &_ => println!("{: >25}: {}", item.name, item.kind),
                    }
                }
            });
            Ok(())
        })
        .unwrap_or(());
        None
    }

    /**
     * Mainly for debugging at this point
     */
    #[allow(dead_code)]
    pub fn list_usb_devices(&self) {
        println!("Listing devices");
        self.execute_command::<commands::response::QomList>(
            commands::Argument::QomList {
                path: "/machine/peripheral",
            },
        )
        .unwrap()
        .items
        .iter()
        .filter(|item| item.kind == "child<usb-host>")
        .for_each(|item| {
            let val = self.execute_command::<serde_json::Value>(
                commands::Argument::QomGet {
                    path: &format!("/machine/peripheral/{}", item.name),
                    property: "hostdevice",
                },
            );
            println!("{}, {:?}", item.name, val);
        });
    }

    pub fn add_device(&self, id: &str, vendor: &i16, product: &i16) {
        self.add_xhci_device();
        info!(
            "Adding device with id {} at {}:{} to {}",
            id, vendor, product, self.vmid
        );
        self.execute_command::<serde_json::Value>(
            commands::Argument::DeviceAdd {
                id: id,
                driver: "usb-host",
                bus: "xhci.0",
                vendorid: Some(&format!("0x{:04}", vendor)),
                productid: Some(&format!("0x{:04x}", product)),
                addr: None,
            },
        )
        .unwrap();
        self.list(&format!("/machine/peripheral/{}", id));
    }

    pub fn remove_device(&self, id: &str) {
        info!("Removing device with id {}", id);
        self.execute_command::<serde_json::Value>(
            commands::Argument::DeviceRemove { id: id },
        )
        .unwrap();
    }
}
