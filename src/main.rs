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
        if let Some(vms) = config.device_mapping.get(&identifier) {
            let mut target_vms: Vec<i32> = vms.to_vec();
            if let Some(default) = config.default_target {
                target_vms.push(default);
            }
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
    // I can make this a lot cleaner
    // for e in events.iter() {
    //     info!("{:?} {}", e.event_type, e);
    //     if let Some(target_vms) = config.device_mapping.get(&e.device_str()) {
    //         info!("Found device owners {:?}", target_vms);
    //         for target_vm in target_vms {
    //             if let Ok(monitor) =
    //                 pve::monitor::QMPMonitor::new(target_vm.to_owned())
    //             {
    //                 match e.event_type {
    //                     libudev::EventType::Add => {
    //                         monitor.add_device(
    //                             &e.get_id(),
    //                             &format!("0x{:04x}", e.vendor),
    //                             &format!("0x{:04x}", e.product),
    //                         );
    //                         info!("Added to {}", target_vm);
    //                         break; // Only break if adding, Always try to remove.
    //                     }
    //                     libudev::EventType::Remove => {
    //                         monitor.remove_device(&e.get_id())
    //                     }
    //                     _ => (), // Ignoring
    //                 }
    //                 break;
    //             }
    //         }
    //     }
    // }
}
