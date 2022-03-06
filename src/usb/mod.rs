#![allow(dead_code)]
pub mod event;

use libudev;
use log::Level::Debug;
use log::{debug, log_enabled};
use std::fmt::{Display, Formatter, Result};
use std::i16;
use std::option::Option;

#[derive(Debug)]
pub struct USBEvent {
    pub event_type: libudev::EventType,
    pub identifier: String,
    pub vendor: i16,
    pub product: i16,
}

impl Display for USBEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "USB[{:04x}:{:04x}]", self.vendor, self.product)
    }
}

impl USBEvent {
    fn get_attribute_from_device(
        device: &libudev::Device,
        attribute: &str,
    ) -> Option<i16> {
        match device.attribute_value(attribute) {
            Some(vender) => vender
                .to_str()
                .map(|id| i16::from_str_radix(id, 16).unwrap()),
            _ => None,
        }
    }

    pub fn get_id(&self) -> String {
        return format!("hotplug-{}", self.identifier.replace("/", "-"));
    }
    pub fn device_str(&self) -> String {
        return format!("{:04x}:{:04x}", self.vendor, self.product);
    }
}

impl From<libudev::Event> for USBEvent {
    fn from(event: libudev::Event) -> Self {
        let device = event.device();
        if log_enabled!(Debug) {
            for property in device.properties() {
                debug!("{:?} = {:?}", property.name(), property.value());
            }
        }
        // This is required for all events
        let product = device
            .property_value("PRODUCT")
            .map(|s| s.to_str().unwrap_or("unknown"))
            .map(|s| String::from(s))
            .expect("Failed to parse product ID from device");

        let parts = match product.split("/").collect::<Vec<&str>>().as_slice() {
            [vendor, product, _] => Some((
                i16::from_str_radix(vendor.to_owned(), 16).unwrap(),
                i16::from_str_radix(product.to_owned(), 16).unwrap(),
            )),
            _ => None,
        };

        USBEvent {
            event_type: event.event_type(),
            identifier: product,
            vendor: parts.unwrap().0,
            product: parts.unwrap().1,
        }
    }
}
