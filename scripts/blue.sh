#!/bin/bash

pactl load-module module-bluetooth-discover
pactl load-module module-switch-on-connect

bluetoothctl << EOF
connect C0:28:8D:93:06:3B
EOF
