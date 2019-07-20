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

The `week` subcommand will give a graphical overview of the past 7 days:

```sh
$ augr week augr
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

## Specifying Dates and Times

The `summary` subcommand has `--start` and `--end` arguments which take a time
that may be specified using several different methods.

### DateTime

DateTimes conforming to [RFC 3339][rfc3339] are allowed. The time zone may be
excluded, and the systems Local time zone will be used.

```sh
$ # What was I doing in the afternoon of July 1st?
$ augr summary --start 2019-07-01T12:00:00 --end 2019-07-01T23:00:00
```

or with a timezone:

```sh
$ # What was I doing in the afternoon of July 1st, in UTC-0500?
$ augr summary --start 2019-07-01T12:00:00-05:00 --end 2019-07-01T23:00:00-05:00
```

[rfc3339]: https://en.wikipedia.org/wiki/ISO_8601

### Date

If you don't the time on a date, it will assume that you mean midnight
(`00:00:00`) in your local timezone. This is useful for getting a summary of
a few days:

```sh
$ # What was I doing in the afternoon on July 4th, 2019, to July 5th, 2019?
$ augr summary --start 2019-07-04 --end 2019-07-06
```

Better yet, you can exclude the current year to make it even shorter:

```sh
$ # What was I doing in the afternoon on July 4th, 2019, to July 5th, 2019?
$ augr summary --start 7-4 --end 7-6
```

### Time

You may also omit the date, and just write the hour and minute.

```sh
$ # What have I done since 16:00 today?
$ augr summary --start 16:00
```

### Duration

Writing a duration will subtract it from the current time. The [`parse_duration`]
[] crate is used, which supports the standard set by [systemd.time][] and more.

```sh
$ # Get a summary of the week
$ augr summary --start 1week
```

[`parse_duration`]: https://crates.io/crates/parse_duration
[systemd.time]: https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing Time Spans
