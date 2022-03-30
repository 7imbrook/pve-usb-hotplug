
run-qemu:
	kvm \
		-smp 4 \
		-m 2G \
		-cpu host \
	 	-machine type=q35 \
		-boot d -cdrom /home/timbrook/Downloads/ubuntu-21.10-desktop-amd64.iso \
		-enable-kvm \
		-chardev 'socket,id=qmp,path=/var/run/qemu-server/101.qmp,server=on,wait=off' \
		-mon 'chardev=qmp,mode=control' \
		-monitor stdio \
		-usb \
		-readconfig ./pve-q35-4.0.cfg

build:
	cargo build --release

deploy-dev: build 
	ssh pve -C systemctl stop hotplug-usb
	scp ./target/release/vm-agent root@pve:/usr/local/bin/vm-agent
	ssh pve -C 'systemctl daemon-reload && systemctl start hotplug-usb'