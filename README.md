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

## Specifying Dates and Times

The `summary` subcommand has `--start` and `--end` arguments which take a time
that may be specified using several different methods.

### DateTime

DateTimes conforming to [RFC 3339][rfc3339] are allowed. The time zone may be
excluded, and the systems Local time zone will be used.

```sh
$ # What was I doing in the afternoon of July 1st?
$ time-tracker summary --start 2019-07-01T12:00:00 --end 2019-07-01T23:00:00
```

or with a timezone:

```sh
$ # What was I doing in the afternoon of July 1st, in UTC-0500?
$ time-tracker summary --start 2019-07-01T12:00:00-05:00 --end 2019-07-01T23:00:00-05:00
```

[rfc3339]: https://en.wikipedia.org/wiki/ISO_8601

### Date

If you don't the time on a date, it will assume that you mean midnight
(`00:00:00`) in your local timezone. This is useful for getting a summary of
a few days:

```sh
$ # What was I doing in the afternoon on July 4th, 2019, to July 5th, 2019?
$ time-tracker summary --start 2019-07-04 --end 2019-07-06
```

Better yet, you can exclude the current year to make it even shorter:

```sh
$ # What was I doing in the afternoon on July 4th, 2019, to July 5th, 2019?
$ time-tracker summary --start 7-4 --end 7-6
```

### Time

You may also omit the date, and just write the hour and minute.

```sh
$ # What have I done since 16:00 today?
$ time-tracker summary --start 16:00
```

### Duration

Writing a duration will subtract it from the current time. The [`parse_duration`]
[] crate is used, which supports the standard set by [systemd.time][] and more.

```sh
$ # Get a summary of the week
$ time-tracker summary --start 1week
```

[`parse_duration`]: https://crates.io/crates/parse_duration
[systemd.time]: https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing Time Spans
