#!/bin/python

import usb.core
from usb.core import USBError
import usb.util

# Find ActionStar/StarTech KVM (SV231DPU2)
dev = usb.core.find(idVendor=0x2101, idProduct=0x1406)

# Found?
if dev is not None:
    for cfg in dev:
        for i in range(cfg.bNumInterfaces):
            if dev.is_kernel_driver_active(i):
                dev.detach_kernel_driver(i)

    # Set the active configuration.
    dev.set_configuration()

    # Magic data via URB_CONTROL out
    bmRequestType = 0x21
    bRequest = 0x9
    wValue = 0x0203
    wIndex = 0x1
    data = [0x03, 0x5c, 0x04, 0x00, 0x00]

    try:
        ret = dev.ctrl_transfer(bmRequestType, bRequest, wValue, wIndex, data)
    except USBError as e:
        print(f'Error sending magic sequence to device {e.strerror}')
else:
    print('Device not found')
