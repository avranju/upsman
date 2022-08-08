# upsman

A small tool for talking to a [Network UPS Tools](https://networkupstools.org) server to do the following:

- Fetch value of following variables:
    - Input voltage
    - Output voltage
    - Output current
    - Power usage in watts
- Send "load on" command which will cause the UPS to start sending power to connected devices.
- Send "load off" command which will cause the UPS to turn off power to connected devices.

Usage instructions can be found through judicious application of the `--help` parameter.