use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std;
use std::fmt::Debug;
use std::fmt::Display;
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

impl Display for QMPMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.vmid)
    }
}

impl QMPMonitor {
    /**
     * Connects and make first contact the the QEMU server
     * Required kvm commands to be able to use this
     *     -chardev 'socket,id=qmp,path=/var/run/qemu-server/101.qmp,server=on,wait=off'
     *     -mon 'chardev=qmp,mode=control'
     */
    pub fn new(id: i32) -> Option<Self> {
        let socket =
            Path::new("/var/run/qemu-server/").join(format!("{}.qmp", id));

        match UnixStream::connect(&socket) {
            Ok(stream) => {
                let qmp = QMPMonitor {
                    vmid: id,
                    stream: stream,
                };
                qmp.init();
                Some(qmp)
            }
            _ => None,
        }
    }

    /**
     * Parse until newline from the qmp socket into the serde type specified
     */
    fn read_message<T>(&self) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        let mut reader = BufReader::new(&self.stream);
        let mut buf = String::default();
        reader.read_line(&mut buf).unwrap();
        debug!("Recieve Raw: {}", buf);
        serde_json::from_str(&buf)
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
    fn execute<T>(&self, command: &commands::QMPMessage) -> T
    where
        T: DeserializeOwned,
    {
        self.send_message(command);
        self.read_message().unwrap()
    }

    fn execute_command<T>(&self, argument: commands::Argument) -> T
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
        ));
        self.add_xhci_device();
    }

    /**
     * While most likely already present on most PVE configured vms you'd want to hotplug
     * this ensures that the xhci bus, which provides the usb 3.0 support, is active.
     */
    fn add_xhci_device(&self) {
        info!("Initalizing xhci device");
        self.execute::<serde_json::Value>(&commands::build_command(
            commands::Argument::DeviceAdd {
                id: "xhci",
                driver: "nec-usb-xhci",
                bus: "pci.1",
                addr: Some("0x1b"),
                vendorid: None,
                productid: None,
            },
        ));
    }

    pub fn add_device(&self, id: &str, vendor: &i16, product: &i16) {
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
        );
    }

    pub fn remove_device(&self, id: &str) {
        info!("Removing device with id {}", id);
        self.execute_command::<serde_json::Value>(
            commands::Argument::DeviceRemove { id: id },
        );
    }
}
