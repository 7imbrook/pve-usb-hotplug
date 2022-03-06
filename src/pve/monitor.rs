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

        let stream = match UnixStream::connect(&socket) {
            Ok(stream) => stream,
            Err(_) => return Err("Unable to connect to VM, not running."),
        };
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
    fn read_message<T>(&self) -> T
    where
        T: DeserializeOwned,
    {
        let mut reader = BufReader::new(&self.stream);
        let mut buf = String::default();
        reader.read_line(&mut buf).unwrap();
        debug!("Recieve Raw: {}", buf);
        serde_json::from_str(&buf)
            .expect(&format!("Failed to process message: {}", buf))
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
        self.read_message()
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
        self.read_message::<serde_json::Value>();
        self.execute::<serde_json::Value>(&commands::build_command(
            commands::Argument::Handshake {},
        ));
    }

    /**
     * Mainly for debugging at this point
     */
    #[allow(dead_code)]
    pub fn list_usb_devices(&self) {
        debug!("Listing devices");
        self.execute_command::<commands::response::QomList>(
            commands::Argument::QomList {
                path: "/machine/peripheral",
            },
        )
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
        info!(
            "Adding device with id {} at {}:{} to {}",
            id, vendor, product, self.vmid
        );
        self.execute_command::<serde_json::Value>(
            commands::Argument::DeviceAdd {
                id: id,
                driver: "usb-host",
                bus: "xhci.0",
                vendorid: &format!("0x{:04}", vendor),
                productid: &format!("0x{:04x}", product),
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
