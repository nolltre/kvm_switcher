# KVM Switcher
Simple program that changes source on a ActionStar/StarTech KVM (SV231DPU2)

The magic byte sequence was retrieved using Wireshark to capture the USB traffic from the Startech official tool and then reimplemented with libusb.

Tested with Linux and MacOs Catalina.

## Prerequisites
Install libusb-1.0

## Linux quirks
Add a udev rule so that you don't have to have superuser privileges to run the programs:  
```shell
$ cat /etc/udev/rules.d/99-startech-kvm.rules 
# Make sure everyone can send commands to the StarTech SV231DPU2
SUBSYSTEM=="usb", ATTRS{idVendor}=="2101", ATTRS{idProduct}=="1406", MODE="0666"
```

## Run
If you have libusb installed and the correct udev in Linux, you can run the Python version directly:  
```shell
python3 kvm_switch.py
```

## Compile
Compile with either `make` or `make release`. The latter just enables the code
optimisations in the GCC compiler. The former will give you debug symbols.
