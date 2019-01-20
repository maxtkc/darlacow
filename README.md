[![yay](https://img.shields.io/badge/cow-moo-ffffff.svg)](http://www.orangefreesounds.com/wp-content/uploads/2016/09/Cow-mooing-loudly.mp3?_=1)

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
> A far-too-complex system for driving a very strange robotic cow.

## User's Guide

### Troubleshooting `Why isn't she working?`

- Darla is powered on `(Hint: Lots of blinky lights)`
- Darla is connected to the internet `(Hint: More blinky lights -- specifically around ethernet port)`
- Darla has been power cycled `(Hint: Have you tried turning it off and on again?)`
- Sam has been emailed `(stkchristy@gmail.com)`
- Max has been emailed `(maxkatzchristy@gmail.com)`

## Developer's Guide

The Raspberry Pi (RPI) is hosting a web server which clients can connect to to edit and play sequences. The RPI is running Raspbian Stretch lite (headless) and starts the server, a compiled executable, upon booting up. The executable is cross compiled from a machine with the Rust compiler installed.

### Building

How to build and install required software from scratch.

- Required tools
  - Raspberry Pi
  - SD Card (4GB+)
  - Computer Running Linux (others will work...)
- Install Raspbian on the SD Card
  - [Download Raspbian](https://www.raspberrypi.org/downloads/raspbian/) (Lite, torrent or zip)
  - Use `lsblk` to find partition name and `dd` to write to disk. `if=~/Downloads/<file here> of=/dev/<partition name (sd_)>`
  - Add `ssh` file to the root of the boot partition of the install
- Build the binary
  - Install [Rust](https://rustup.rs/)
  - Set up [cross compiling](https://github.com/japaric/rust-cross) for `armv7` (`gnueabihf` is good)
    - Basically just install the gcc-for-arm-thingy, add the target, edit the config
  - Clone the repo and `cd` into it
  - `cargo build --target=armv7-unknown-linux-gnueabihf`
  - Copy the files over
    - `scp -r templates static target/armv7-unknown-linux-gnueabihf/debug/darlacow pi@<ip>:~`
  - `ssh` to server and test run
- Make it automatically run every time
  - `sudo vim /etc/rc.local`
  - Add between comment and exit 0
    - `ROCKET_ENV=production cd /home/pi && ./darlacow`

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

## Bluetooth
> oof

Make sure the pi user is added to all relevant user groups (otherwise sudo is necessary)
Other issue with module stuff, [answer](https://raspberrypi.stackexchange.com/questions/67617/bluetoothctl-fails-to-connect-to-any-device-failed-to-connect-org-bluez-erro)
Essentially:
- `sudo vi /etc/pulse/default.pa`
  - Comment out the line loading `module-bluetooth-discover` to `#load-module module-bluetooth-discover`
- `sudo vi /usr/bin/start-pulseaudio-x11`
  - after the lines 

```
    if [ x”$SESSION_MANAGER” != x ] ; then
        /usr/bin/pactl load-module module-x11-xsmp “display=$DISPLAY session_manager=$SESSION_MANAGER” > /dev/null
    fi
```

  - add line

```
    /usr/bin/pactl load-module module-bluetooth-discover
```

  - Bluetooth should now be loaded after x11 is started
  - Reboot
  - `pactl load-module module-bluetooth-discover`

### TODO

- [ ] Combine and make it function properly
  - [x] GPIO
  - [ ] Serial
  - [ ] Bluetooth
- [ ] Use [rust-embed](https://github.com/pyros2097/rust-embed) to make deployment easier
- [ ] Add theme
- [ ] Cow over moon progress bar?

## Authors

👴 [Sam Christy](mailto:stkchristy@gmail.com) 👴

🤣 [Max Katz-Christy](mailto:maxkatzchristy@gmail.com) 🤣

🤔 [Nina Katz-Christy](mailto:ninakatzchristy@college.harvard.edu) 🤔
