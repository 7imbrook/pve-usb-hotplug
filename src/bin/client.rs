use vm_agent::pve::monitor::QMPMonitor;

fn main() {
    println!("Connecting to client");
    let monitor = QMPMonitor::new(101).expect("Failed to connect to client VM");
}
