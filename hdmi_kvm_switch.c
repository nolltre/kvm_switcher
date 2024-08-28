/**
 * File name:	kvm_switch.c
 * Copyright:   Daniel Karmark
 * Created:		2024-08-27
 * Modified:	2024-08-27
 * Description: Change KVM source on StarTech KVM (SV211HDUA)
 **/

#include <stdio.h> // printf
#include <wchar.h> // wchar_t
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <libusb-1.0/libusb.h>

#define VID 0x10d5
#define PID 0x55a2
#define CONFIGURATION 1

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <libusb-1.0/libusb.h>

#if !defined VID || !defined PID
#error "VID and PID need to be set by defining SV231DPU2 or SV211HDUA"
#else

libusb_device_handle *hndl;

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

    rc = libusb_hotplug_register_callback(NULL,
            LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT, 0, VID,
            PID,LIBUSB_HOTPLUG_MATCH_ANY, hotplug_callback_detach, NULL, &hp);
    if (LIBUSB_SUCCESS != rc) {
        fprintf (stderr, "Error registering callback\n");
        libusb_exit (NULL);
        return EXIT_FAILURE;
    }

    libusb_exit(NULL);
    hndl = libusb_open_device_with_vid_pid(NULL, VID, PID);
    if (hndl == NULL) {
        printf("Error opening device\n");
        return EXIT_FAILURE;
    }

    // Get number of interfaces and detach the kernel driver
    struct libusb_config_descriptor *cfg;
    libusb_get_active_config_descriptor(libusb_get_device(hndl), &cfg);

    for (int i = 0; i < cfg->bNumInterfaces; ++i)
        if (libusb_kernel_driver_active(hndl, i))
            libusb_detach_kernel_driver(hndl, i);


    // Try to set the configuration (defined above)
    if (libusb_set_configuration(hndl, CONFIGURATION) != 0) {
        printf("Error setting configuration\n");
    }

    // Send HID request as an interrupt transfer
    // The HDMI switch sends a command to the output endpoint on interface 1
    // Find the output endpoint
    // Send magic data
    unsigned char data[] = {0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00};
    /* uint8_t  	bmRequestType = 0x21; */
    /* uint8_t  	bRequest = 0x9; */
    /* uint16_t  	wValue = 0x0203; */
    /* uint16_t  	wIndex = 1; */
    /* uint16_t  	wLength = 5; */
    /* libusb_control_transfer(hndl, bmRequestType, bRequest, wValue, wIndex, data, wLength, 0); */

    // Clean-up
    if (cfg) libusb_free_config_descriptor(cfg);
    if (hndl) libusb_close(hndl);

    libusb_exit(NULL);
    return EXIT_SUCCESS;
}
#endif /* ifndef VID */
