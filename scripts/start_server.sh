#!/bin/bash

killall plymouthd

# AUDIO MUST NOT BE ROOT

su -c "/home/pi/darlacow/scripts/start_audio.sh" pi &

stty -F /dev/ttyACM0 cs8 9600 ignbrk -brkint -icrnl -imaxbel -opost -onlcr -isig -icanon -iexten -echo -echoe -echok -echoctl -echoke noflsh -ixon -crtscts

# SERVER MUST BE ROOT

cd /home/pi/darlacow && ROCKET_ENV=staging ./darlacow > server_logs
