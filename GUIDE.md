## User Guide

This guide is intended to get a user up and running with `augr`, and also
describe some of it's more advanced features. It assumes that the `augr` binary
is installed and that the user is familiar with the command line.

### Table of Contents

* [Configuration](#configuration)
* [Basics](#basics)

### Configuration

`augr` is a program to track how your time is being spent. It is designed to be
used on multiple devices, using [Syncthing][] to synchronize it's data. Each
device that it is used on must be configured before it can be used.

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
