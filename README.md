# PVE USB Hotplug Daemon
Listens for new devices via libudev and uses the qemu qmp interface to add them to running VMs

## Why?
My home usecase for proxmox is to run two gaming/daily driver vms. I want to be able to plug devices in without thinking about it and I don't want to add a USB PCI card to solve this problem.
