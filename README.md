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

### Initial Setup of Raspberry Pi

- Required tools
  - Raspberry Pi
  - SD Card (4GB+)
  - Computer Running Linux (others will work...)
- Install Raspbian on the SD Card
  - [Download Raspbian](https://www.raspberrypi.org/downloads/raspbian/) (Lite, torrent or zip)
  - Use `lsblk` to find partition name and `dd` to write to disk. `if=~/Downloads/<file here> of=/dev/<partition name (sd_)>`
  - Add `ssh` file to the root of the boot partition of the install
- Make the Raspberry Pi run the web server executable every time the Raspberry Pi boots up:
  - On the Raspberry Pi, edit the file `/etc/rc.local` and add the following:
    ```
    # Log the date and time of the boot.
    date >> /home/pi/darlacow/startup_logs
    # Run Darla's web server executable and write the output to a log file.
    /home/pi/darlacow/scripts/start_server.sh >> /home/pi/darlacow/startup_logs &
    ```

### How to Update the Webserver Code

- On your developer machine (not Darla's Raspberry Pi), make sure you have the most up-to-date version of the Darla web server source code.
  - Clone the git repository, if you have not already (see the official [GitHub instructions](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository) for more details):
    ```
    git clone https://github.com/maxtkc/darlacow.git
    cd darlacow
    ```
  - Pull the latest git repo files
    ```
    git pull
    ```
- Now, make your desired changes to the git repo files.
- Push your changes to the git repo (see official [GitHub instructions](https://docs.github.com/en/get-started/using-github/github-flow#make-changes) for more details):
  ```
  git add --all
  # Replace MESSAGE with a short description of the change you made.
  git commit -m "MESSAGE"
  # Your GitHub account will need write permissions to the Darla git repo in order to run this command.
  git push
  ```
- Gain access to the Raspberry Pi via one of the following options:
  - Option 1: Use SSH to gain remote terminal access to the Raspberry Pi.
    - Run the SSH command from your developer machine terminal:
      ```
      ssh pi@darlacow.asuscomm.com
      ```
    - Type the password, if prompted.
  - Option 2: Connect a monitor, keyboard, and mouse to the Raspberry Pi.
    - A portable USB-C monitor, and a wireless keyboard+mouse combo are in Sam's Louis Armstrong room in a small Amazon box.
- Pull your changes onto the Raspberry Pi:
  - Navigate to the git repo and pull your changes:
    ```
    cd /home/pi/darlacow
    git pull
    ```
  - Confirm that your changes have been pulled down. You should see your commit message from earlier at the top of the git log:
    ```
    git log
    # Press Q or ESC once you have confirmed your commit is present in the log.
    ```
- Build the code and install the new executable so that it will get run when the Raspberry Pi is booted:
  ```
  # This takes between 4 to 40 minutes, depending on how big your code change is:
  ./scripts/build_and_install.sh
  ```
- Check that your code changes had the expected effect:
  - Reboot the Raspberry Pi so that the new code is used to launch the web server:
    ```
    sudo reboot
    ```
  - Give the Pi 1-3 minutes to reboot and start up the web server. You can check the web server logs on the Pi for any info or errors at `/home/pi/darlacow/server_logs`.
  - Connect to Darla's web server on your phone or developer machine to see that your change is as-expected: http://darlacow.asuscomm.com/


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
- LED Matrix
  - For info on the big ceiling-mounted LED matrix, see [the LED matrix README](led_matrix/README.md).


### Additional Notes

- `/etc/rc.local` on the Raspberry Pi tells the pi to start the web server on boot.

### Things we write down

#### Bluetooth
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

Auto pairing:
[https://raspberrypi.stackexchange.com/questions/53408/automatically-connect-trusted-bluetooth-speaker](https://raspberrypi.stackexchange.com/questions/53408/automatically-connect-trusted-bluetooth-speaker)

- Just add `/home/pi/darlacow/scripts/start_server.sh` to `/etc/rc.local`

#### MPD

MPD is started from `start_audio.sh`
After installing, `sudo systemctl disable mpd`

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
