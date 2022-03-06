use libudev::{Context, EventType, Monitor};
use log::{debug, info};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub fn start_listener() -> Receiver<super::USBEvent> {
    let (tx, rx) = mpsc::channel();
    let _listen = thread::spawn(move || monitor_socket(tx));
    info!("Started udev monitor");
    // Do this later probably, but for now, eh
    // listen.join();
    rx
}

fn monitor_socket(sender: Sender<super::USBEvent>) {
    let context = Context::new().unwrap();
    let mut monitor = Monitor::new(&context).unwrap();
    monitor
        .match_subsystem_devtype("usb", "usb_device")
        .unwrap();

    let mut socket = monitor.listen().unwrap();
    loop {
        let raw_event = match socket.receive_event() {
            Some(e) => e,
            None => {
                // receive_event doesn't block, delay here
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let event = match raw_event.event_type() {
            EventType::Remove | EventType::Add => {
                Some(super::USBEvent::from(raw_event))
            }
            // Not handling non add or remove
            _ => None,
        };
        debug!("{:?}", event);
        if let Some(e) = event {
            sender.send(e).expect("Failed to send event");
        }
    }
}
