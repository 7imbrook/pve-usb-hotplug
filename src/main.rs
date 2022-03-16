mod conf;
mod pve;
mod usb;

use env_logger;
use env_logger::Env;
use log;
use log::{info, warn};

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

fn main() {
    configure_logging();
    splash();

    // This needs to be reloadable
    let config = conf::configure_config();

    // Spawns a new thread that publishes USBEvents to events
    usb::event::start_listener().iter().for_each(|event| {
        let identifier = event.device_str();
        if let Some(target_vms) = config.device_mapping.get(&identifier) {
            let monitors = target_vms
                .iter()
                .map(|vid| pve::monitor::QMPMonitor::new(*vid))
                .filter(|m| m.is_ok())
                .map(|m| m.unwrap())
                .collect::<Vec<pve::monitor::QMPMonitor>>();
            if monitors.len() > 1 {
                warn!("Multiple online targets, asigning to first one");
            }
            for m in monitors {
                match event.event_type {
                    libudev::EventType::Add => {
                        m.add_device(
                            &event.get_id(),
                            &event.vendor,
                            &event.product,
                        );
                        break;
                    }
                    libudev::EventType::Remove => {
                        m.remove_device(&event.get_id())
                    }
                    _ => (),
                }
            }
        } else if event.event_type == libudev::EventType::Add {
            warn!("No targets for device {}", event);
        }
    });
}
