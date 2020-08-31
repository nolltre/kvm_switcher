/**
 * File name:	kvm_switch.c
 * Copyright:   Daniel Karmark
 * Created:		2018-06-17
 * Modified:	2020-08-31
 * Description: Change KVM source on ActionStar/StarTech KVM (SV231DPU2)
 **/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <libusb-1.0/libusb.h>

#define VID 0x2101
#define PID 0x1406

libusb_device_handle* hndl;

static int LIBUSB_CALL hotplug_callback_detach(libusb_context *ctx,
        libusb_device *dev, libusb_hotplug_event event, void *user_data)
{
    (void)ctx;
    (void)dev;
    (void)event;
    (void)user_data;

    printf ("Device detached\n");

    if (hndl) {
        libusb_close (hndl);
        hndl = NULL;
        exit(0);
    }

    return 0;
}

int main()
{
    int rc;
    libusb_hotplug_callback_handle hp;

    rc = libusb_init(NULL);
    if (rc < 0) {
        printf("Unable to initialise libusb\n");
        return EXIT_FAILURE;
    }

    if (!libusb_has_capability (LIBUSB_CAP_HAS_HOTPLUG)) {
        printf ("Hotplug capabilities are not supported on this platform\n");
        libusb_exit (NULL);
        return EXIT_FAILURE;
    }

    rc = libusb_hotplug_register_callback(NULL, LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT, 0, VID,
            PID,LIBUSB_HOTPLUG_MATCH_ANY, hotplug_callback_detach, NULL, &hp);
    if (LIBUSB_SUCCESS != rc) {
        fprintf (stderr, "Error registering callback 1\n");
        libusb_exit (NULL);
        return EXIT_FAILURE;
    }

    hndl = libusb_open_device_with_vid_pid(NULL, VID, PID);
    if (hndl == NULL) {
        printf("Error opening device\n");
        return -1;
    }

    // Get number of interfaces and detach the kernel driver
    struct libusb_config_descriptor * cfg;
    libusb_get_active_config_descriptor(libusb_get_device(hndl), &cfg);

    for (int i = 0; i < cfg->bNumInterfaces; ++i)
        if (libusb_kernel_driver_active(hndl, i))
            libusb_detach_kernel_driver(hndl, i);


    // Try to set the configuration (1)
    if (libusb_set_configuration(hndl, 1) != 0) {
        printf("Error setting configuration\n");
    }

    // Send magic data
    unsigned char data[] = {0x03, 0x5c, 0x04, 0x00, 0x00};
    uint8_t  	bmRequestType = 0x21;
    uint8_t  	bRequest = 0x9;
    uint16_t  	wValue = 0x0203;
    uint16_t  	wIndex = 1;
    uint16_t  	wLength = 5;
    libusb_control_transfer(hndl, bmRequestType, bRequest, wValue, wIndex, data, wLength, 0);

    if (cfg) libusb_free_config_descriptor(cfg);
    if (hndl) libusb_close(hndl);

    libusb_exit(NULL);
    return EXIT_SUCCESS;
}
