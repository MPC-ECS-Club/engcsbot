# EngCS Bot

# Commands
| Feature           | Status | Note                                                                                    |
|-------------------|--------|-----------------------------------------------------------------------------------------|
| `/info`           | ✅      | View information about the bot                                                          |
| `/upcoming`       | ✅      | View upcoming meetings for this week                                                    |
| *`/announce`      | ✅      | Send a manual announcement with some formatting (sends as an embed)                     |
| *`/schedule`      | ✅      | Schedule a new weekly meeting                                                           |
| *`/cancelday`     | ✅      | Cancel an entire day of meetings.                                                       |
| *`/removemeeting` | ✅      | Remove a specific meeting entirely                                                      |
| `/jsonembed`      | ✅      | Allows you to create a custom embed with many options (**todo**: documentation missing) |
| *`/shutdown`      | ✅      | Shutdown the bot if necessary                                                           |

\* These commands require adminstrator permission to run.

# TODO list
- Lot's of `unwrap()` calls. Clean this up
- Give `ScheduledMeeting` a UID, so that it  can be tracked easier.
- Add ability to attach a 'note' to a meeting that will be posted alongside an automatic announcement.

## Running normally
The following will run the bot in release mode with the given token
I recommend using a separate token/bot when running in debug mode so that
the real bot can remain active at all times.
<br>
`RUST_BACKTRACE=1 DISCORD_TOKEN=token cargo run -r`

## Building for Raspberry PI 4 Model B
My personal setup uses a raspberr pi 4 model B, running without
the desktop environment loaded an SSH setup for remote usage.
Once SSH'd into the rasp-pi, make a folder for the bot.

On your development computer install `cross` (`cargo install cross`) and its dependencies (such as docker, see docs for cross) and the
target for the raspberry pi `rustup target add aarch64-unknown-linux-gnu`
You may also need to install other dependencies, on Arch Linux for example, `pacman -S aarch64-linux-gnu-gcc` is necessary.
Cross should handle these dependencies though, so just try compiling with cross first.

Once compiled (ideally in release mode), transfer the binary to the raspberry pi via ssh <br> 
(`scp target/aarch64-unknown-linux-gnu/release/engcsbot pi-hostname@pi-ipaddr:~/path-to-bot/engcsbot`)

Once on the raspberry pi, ensure it is executable `chmod +x ./engcsbot`

I recommend also transferring a file named `token.txt` that contains the bot token onto the raspberry pi in the same folder.

## Running on Raspberry PI 4 Model B
Assuming you have completed the steps above I recommend creating a
bash script named `engcsbot-launch` and edit it to contain the following:
```bash
#!/usr/bin/env bash

RUST_BACKTRACE=1 DISCORD_TOKEN=$(cat prodtoken.txt) nohup ./engcsbot > log.txt 2>&1 &
```
Then ensure the script is executable `chmod +x engcsbot-launch`. By running this script, it will launch the bot in the background
and write to `log.txt` for logs. You can now safely disconnect from SSH without turning off the bot.
If you wish to shut down the bot, you can do `kill $(pidof engcsbot)` to do so.
If you want to ensure the bot starts on raspberry pi startup, you can create a service, but you can google that yourself.
