# Time Tracker

`time-tracker` is a simple time tracking app, that is designed for one person
to use across multiple devices.

This project is heavily inspired by [timewarrior][].

## Setup

Build the application:

```sh
$ git clone https://github.com/Geemili/time-tracker
$ cd time-tracker
$ cargo build
```

Create and edit the config file (located at `~/.config/time-tracker/config.toml`
on linux):

```toml
sync_folder = "/some/sync/folder"
device_id = "laptop"
```

`sync_folder` should be a synchronized between devices by a service like
[Syncthing][] or dropbox.

`device_id` should be unique for all devices that use the same sync folder.

## Usage

Simply running `time-tracker` will give a summary of the day. With nothing
tracked so far, only some headers will be output:

```sh
$ time-tracker
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
```

The `start` subcommand is used to begin tracking your time:

```sh
$ time-tracker start test
$ time-tracker
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 15:41 0m       0m        test
```

We can filter the output of the summary by giving some tags to the `summary`
subcommand:

```sh
$ time-tracker summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 06:33 54m      54m       blog time-tracker
      07:27 47m      1h 41m    blog
      08:15 2h 51m   4h 33m    social-media
      11:06 29m      5h 2m     food
      11:36 1h 38m   6h 41m    coding time-tracker
      13:15 1h 31m   8h 12m    coding time-tracker
      14:46 8m       8h 20m    social
      14:54 53m      9h 14m    time-tracker
$ time-tracker summary time-tracker
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 06:33 54m      54m       blog time-tracker
      11:36 1h 38m   2h 33m    coding time-tracker
      13:15 1h 31m   4h 4m     coding time-tracker
      14:54 53m      4h 57m    time-tracker
```

The `week` subcommand will give a graphical overview of the past 7 days:

```sh
$ time-tracker week time-tracker
Day 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 
Sun                                                                         
Mon                                                                         
Tue                                                                         
Wed                                                                         
Thu                                                                         
Fri                                                        ██████           
Sat                     ███            █████████████                        
```

## Using with Termux on Android

Build and upload the android executable:

```sh
$ nix-shell android.nix --run "cargo build --target=armv7-linux-androideabi"

# Upload the android executable using adb
$ adb push target/armv7-linux-androideabi/time-tracker /storage/self/primary/
```

Then in termux on your phone:

```sh
$ cp /storage/self/primary/time-tracker ./
$ chmod ./time-tracker
$ ./time-tracker
```

[timewarrior]: https://taskwarrior.org/docs/timewarrior/index.html
[Syncthing]: https://syncthing.net/
