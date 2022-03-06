
build:
	cargo build

sync: 
	rsync --progress -r ./target/debug/vm-agent root@pve:~/target/debug/vm-agent

reload: 
	ssh pve -C killall vm-agent

run_example:
	qemu-system-x86_64 -chardev 'socket,id=qmp,path=/home/timbrook/test.qmp,server=on,wait=off' -mon 'chardev=qmp,mode=control' -device 'nec-usb-xhci,id=xhci,bus=pci.0,addr=0x1b'
