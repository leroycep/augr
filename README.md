# augr

`augr` tracks your time, generates summaries, and syncs between devices using
[Syncthing][].

This project is heavily inspired by [timewarrior][].

## Useful Links

- [Official Matrix Chatroom](https://matrix.to/#/!RMSJfJyCNLxdOzbmQG:geemili.xyz?via=geemili.xyz&via=matrix.org)

## Setup

Build the application:

```sh
$ git clone https://github.com/Geemili/augr
$ cd augr
$ cargo build
```

Create and edit the config file (located at `~/.config/augr/config.toml`
on linux):

```toml
sync_folder = "/some/sync/folder"
device_id = "laptop"
```

`sync_folder` should be a synchronized between devices by a service like
[Syncthing][] or dropbox.

`device_id` should be unique for all devices that use the same sync folder.

## Usage

Simply running `augr` will give a summary of the day. With nothing
tracked so far, only some headers will be output:

```sh
$ augr
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
```

The `start` subcommand is used to begin tracking your time:

```sh
$ augr start test
$ augr
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 15:41 0m       0m        test
```

We can filter the output of the summary by giving some tags to the `summary`
subcommand:

```sh
$ augr summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 06:33 54m      54m       blog augr
      07:27 47m      1h 41m    blog
      08:15 2h 51m   4h 33m    social-media
      11:06 29m      5h 2m     food
      11:36 1h 38m   6h 41m    coding augr
      13:15 1h 31m   8h 12m    coding augr
      14:46 8m       8h 20m    social
      14:54 53m      9h 14m    augr
$ augr summary augr
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 06:33 54m      54m       blog augr
      11:36 1h 38m   2h 33m    coding augr
      13:15 1h 31m   4h 4m     coding augr
      14:54 53m      4h 57m    augr
```

The `chart` subcommand will give a graphical overview of the past 7 days:

```sh
$ augr chart augr
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
$ adb push target/armv7-linux-androideabi/augr /storage/self/primary/
```

Then in termux on your phone:

```sh
$ cp /storage/self/primary/augr ./
$ chmod ./augr
$ ./augr
```

[timewarrior]: https://taskwarrior.org/docs/timewarrior/index.html
[Syncthing]: https://syncthing.net/
