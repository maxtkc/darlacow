# Darla the Cow

```
_______________________
< Hi, my name is Darla! >
 -----------------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

## tl;dr
A far-too-complex system for driving a very strange robotic cow.

## User's Guide

### Troubleshooting `Why isn't she working?`

+ Darla is powered on `(Hint: Lots of blinky lights)`
+ Darla is connected to the internet `(Hint: More blinky lights -- specifically around ethernet port)`
+ Darla has been power cycled `(Hint: Have you tried turning it off and on again?)`
+ Sam has been emailed `(stkchristy@gmail.com)`
+ Max has been emailed `(maxkatzchristy@gmail.com)`

## Developer's Guide

The Raspberry Pi (RPI) is hosting a web server which clients can connect to to edit and play sequences. The RPI is running Raspbian Stretch lite (headless) and starts the server, a compiled executable, upon booting up. The executable is cross compiled from a machine with the Rust compiler installed.

### Components

- Raspberry Pi
  - Raspbian Stretch Lite
  - Runs compiled binary on boot
  - Connected to 192.168.1.17 for TODO (eth/wifi)
  - Connected to 192.168.1.18 for TODO (eth/wifi)
- Rust
  - Programming Language
  - Used to build the binary
  - Crates (packages)
    - `Rocket` web-engine
    - `tera` template engine

### Things we write down

TODO

## Authors

👴 [Sam Christy](mailto:stkchristy@gmail.com) 👴

🤣 [Max Katz-Christy](mailto:maxkatzchristy@gmail.com) 🤣

🤔 [Nina Katz-Christy](mailto:ninakatzchristy@college.harvard.edu) 🤔
