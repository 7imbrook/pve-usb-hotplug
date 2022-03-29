mod conf;
mod pve;
mod usb;

use env_logger;
use env_logger::Env;
use log;
use log::info;
use pve::monitor::QMPMonitor;
use usb::USBEvent;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn configure_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
}

fn splash() {
    info!("Starting USB Hotplug for proxmox");
    info!("┌────────────────────────────────┐");
    info!("│    _   ,--()                   │");
    info!("│ --'-.------|>     Timbrook     │");
    info!("│     `--[]                      │");
    info!("└────────────────────────────────┘");
}

fn build_monitors(target_vms: Vec<i32>) -> Vec<QMPMonitor> {
    target_vms
        .iter()
        .map(|vid| pve::monitor::QMPMonitor::new(*vid))
        .filter(|m| m.is_ok())
        .map(|m| m.unwrap())
        .collect::<Vec<pve::monitor::QMPMonitor>>()
}

fn find_vids_for(identifier: String) -> Vec<i32> {
    let config = conf::configure_config();
    if let Some(vms) = config.device_mapping.get(&identifier) {
        return vms.to_vec();
    }
    return config
        .default_target
        .and_then(|t| Some(vec![t]))
        .or(Some(vec![]))
        .unwrap()
        .to_vec();
}

fn handle_event(event: USBEvent) {
    let identifier = event.device_str();
    let target_vms = find_vids_for(identifier);
    let monitors = build_monitors(target_vms);
    for m in monitors {
        match event.event_type {
            libudev::EventType::Add => {
                m.add_device(&event.get_id(), &event.vendor, &event.product);
                break;
            }
            libudev::EventType::Remove => m.remove_device(&event.get_id()),
            _ => (),
        }
    }
}

fn main() {
    configure_logging();
    splash();
    info!("Build version: {}", VERSION);

    // Spawns a new thread that publishes USBEvents to events
    usb::event::start_listener().iter().for_each(|event| {
        handle_event(event);
    });
}
