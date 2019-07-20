# augr

`augr` tracks your time, generates summaries, and syncs between devices using
[Syncthing][].

This project is heavily inspired by [timewarrior][].

## Useful Links

- [Official Matrix Chatroom](https://matrix.to/#/!RMSJfJyCNLxdOzbmQG:geemili.xyz?via=geemili.xyz&via=matrix.org)
- [User Guide](GUIDE.md)

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

Once you have saved the config file, you can run `augr`:

```sh
$ augr
Date  Start Duration Total     Tags
――――― ――――― ―――――――― ――――――――  ――――――――
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
