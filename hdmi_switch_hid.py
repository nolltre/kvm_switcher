#!/usr/bin/env python3

import usb
import sys

vid = 0x10D5
pid = 0x55A2

dev = usb.core.find(idVendor=vid, idProduct=pid)

if dev is None:
    print("Device not found", file=sys.stderr)
    sys.exit(1)

if dev.is_kernel_driver_active(0):
    print("detaching kernel driver")
    dev.detach_kernel_driver(0)

try:
    dev.set_configuration(1)
except Exception as e:
    print("set_configuration", e)


# get an endpoint instance
cfg = dev.get_active_configuration()

# Interface 1 has the ENDPOINT_OUT
intf = cfg[(1, 0)]
ep_out = usb.util.find_descriptor(
    intf,
    # match the first IN endpoint
    custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress)
    == usb.util.ENDPOINT_OUT,
)
assert ep_out is not None

# Write a HID command to the device
hid_data = bytearray([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])

# Send the command to the endpoint
ep_out.write(hid_data)
