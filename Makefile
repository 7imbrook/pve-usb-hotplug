
run-qemu:
	kvm \
		-smp 4 \
		-m 2G \
		-cpu host \
	 	-machine type=q35 \
		-enable-kvm \
		-chardev 'socket,id=qmp,path=/var/run/qemu-server/101.qmp,server=on,wait=off' \
		-mon 'chardev=qmp,mode=control' \
		-readconfig ./pve-q35-4.0.cfg