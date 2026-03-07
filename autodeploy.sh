#!/usr/bin/env bash

hostname="$1"

function fatal() {
  echo -e "$1"
  exit
}


if [[ $# -eq 0 ]]; then
  fatal "Usage: ./autodeploy.sh <ssh hostname>\nExample: ./autodeploy.sh pi@raspberrypi.local\nNote: this script was designed for deploying to a raspberry pi.\ncross, ssh, and sshpass are required"
fi

read -r -s -p "Enter SSH password (for uploading after compilation): " password
echo

echo "Compiling";

cross build --release --target=aarch64-unknown-linux-gnu || fatal "Compilation failed"

echo "Uploading binary"
SSHPASS="$password" sshpass -e scp target/aarch64-unknown-linux-gnu/release/engcsbot "$hostname":~/discordbots/engcsbot/update \
  || fatal "Upload failed"

echo "Stopping the bot"
SSHPASS="$password" sshpass -e ssh "$hostname" "sudo systemctl stop engcsbot" \
  || fatal "failed to stop the bot remotely"

echo "Making a backup of the old executable"
SSHPASS="$password" sshpass -e ssh "$hostname" "cp ~/discordbots/engcsbot/engcsbot ~/discordbots/engcsbot/engcsbot-backup" \
  || fatal "failed to make backup of the existing executable"

echo "Updating"
SSHPASS="$password" sshpass -e ssh "$hostname" "mv ~/discordbots/engcsbot/update ~/discordbots/engcsbot/engcsbot" \
  || fatal "failed to update the executable"

SSHPASS="$password" sshpass -e ssh "$hostname" "chmod +x ~/discordbots/engcsbot/engcsbot" \
  || fatal "failed to make new version executable"


#echo "Starting the bot"
#SSHPASS="$password" sshpass -e ssh "$hostname" "sudo systemctl start engcsbot" \
#  || fatal "failed to start updated bot"

echo "Done"

