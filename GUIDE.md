## User Guide

This guide is intended to get a user up and running with `augr`, and also
describe some of it's more advanced features. It assumes that the `augr` binary
is installed and that the user is familiar with the command line.

### Table of Contents

* [Configuration](#configuration)
* [Basics](#basics)
* [Fixing Mistakes](#fixing-mistakes)
* [Specifying Dates and Times](#specifying-dates-and-times)

### Configuration

`augr` is a program to track how your time is being spent. It is designed to be
used on multiple devices, using [Syncthing] to synchronize it's data. Each
device that it is used on must be configured before it can be used.

[Syncthing]: https://syncthing.net/

### Basics

Once `augr` has been setup, you can track your time. Let's start by tracking
the time you spend reading this guide:

```sh
$ augr start reading
```

Here you are telling `augr` that you have started reading. `augr` will log that
you started "reading" at the current time. We can check that this is indeed the
case with `augr summary`:

```sh
$ augr summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
07/20 13:00 0m       0m        reading
```

Augr maintains a continuous stream of events. Each event ends when the next one
begins. The canonical way to stop tracking task(s) is to start an event with no
tags, like so:

```sh
$ augr start
$ augr summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
07/20 13:00 30m      30m       reading
07/20 13:30 0m       30m       
```

This is no different than any other event, except that the output of `augr chart`
will output blank spaces instead of filled in marks:

```sh
$ augr chart
Day 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 
Tue ████████████████  █████████████████████████████████████████  ███████████
Wed ████████████████ █████████████████████ ███████████████     ██   ████████
Thu ████████████████    ██ ██████████████████████                           
Fri ████████████████     ███████████████████████████ ███████████████████████
Sat ███████████████████████████████       ████  ████████████████████████████
Sun ████████████████████████████         ███      ██████████████    ████████
Mon ████████████████  ██████████████████████████████████████               
```

You can filter the output of the summary by giving some tags to the `summary`
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
$ # Filter the output
$ augr summary augr
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
06/29 06:33 54m      54m       blog augr
      11:36 1h 38m   2h 33m    coding augr
      13:15 1h 31m   4h 4m     coding augr
      14:54 53m      4h 57m    augr
```

If you forget to start tracking for a couple of minutes, you can use the
`--time` option to set the event at a past time.

```sh
$ augr start hello world --time 10min
```

See [Specifying Dates and Times](#specifying-dates-and-times) for a complete
list of ways to specify datetimes.

### Fixing Mistakes

`augr` also supports modifying past events. For example say you started reading,
but then decided that you wanted to tag this reading with `entertainment`. You
can do this with the `tag` subcommand.

```sh
$ augr summary --refs
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
08/31 17:04 14m      14m      reading fbb4d730-c52a-450f-b920-78b20f8209bd
$ augr tag fbb4d730-c52a-450f-b920-78b20f8209bd entertainment
$ augr summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
08/31 17:04 17m      17m      entertainment reading
```

The `--refs` option gives you the `EventRef` of each event. You can then use
that reference to `tag` the event, or change its start time.

```sh
$ augr set-start fbb4d730-c52a-450f-b920-78b20f8209bd 17:15
$ augr summary
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
08/31 17:15 10m      10m      entertainment reading
```

### Specifying Dates and Times

The `summary` subcommand has `--start` and `--end` arguments which take a time
that may be specified using several different methods.

#### DateTime

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

#### Date

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

#### Time

You may also omit the date, and just write the hour and minute.

```sh
$ # What have I done since 16:00 today?
$ augr summary --start 16:00
```

#### Duration

Writing a duration will subtract it from the current time. The [`parse_duration`]
crate is used, which supports the standard set by [systemd.time] and more.

```sh
$ # Get a summary of the week
$ augr summary --start 1week
```

[`parse_duration`]: https://crates.io/crates/parse_duration
[systemd.time]: https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans
