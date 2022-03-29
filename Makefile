
run-qemu:
	qemu-system-x86_64 \
		-smp 4 \
	 	-machine type=q35 \
		-enable-kvm \
		-chardev 'socket,id=qmp,path=/home/timbrook/test.qmp,server=on,wait=off' \
		-mon 'chardev=qmp,mode=control' \
		-boot d -cdrom /home/timbrook/Downloads/ubuntu-21.10-desktop-amd64.iso \
		-m 2G 