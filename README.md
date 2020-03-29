nbdserve
---

Simple network block device server. Created because of usual nbd-server is tricky-ish to cross-compile. Also it typically requires config file, which is inconvenient for one-off use.

There are pre-built versions for Linux, Mac, Windows and Android on [releases page](https://github.com/vi/nbdserve/releases/).

IPv6 is supported, use options like `-a [::1]`.

```
nbdserve 0.1.0
Vitaly "_Vi" Shukela <vi0oss@gmail.com>
Simple network block device server

USAGE:
    nbdserve [FLAGS] [OPTIONS] <file>

FLAGS:
    -h, --help          Prints help information
    -q, --quiet         Quiet mode, suppress non-error output
    -r, --read-only     Read-only mode
        --resize        Support RESIZE NBD extension (not implemented)
        --rotational    Hint clients that elevator algorithm should be used
        --trim          Convert TRIM operations to FALLOC_FL_PUNCH_HOLE or something (not implemented)
    -V, --version       Prints version information

OPTIONS:
    -a, --addr <host>    Address to listen the port on [default: 127.0.0.1]
    -p, --port <port>    TCP port to listen [default: 10809]
    -s, --size <size>    

ARGS:
    <file>    File or device to be served
```

# Windows
On windows, you can use with physical drives like so:

List physical drives:
```
wmic diskdrive list brief
```

Build and run (note that physical drives seem to only work in read only mode):
```
wmic diskdrive list brief
cargo build --release
.\target\release\nbdserve.exe --addr 0.0.0.0 \\.\PHYSICALDRIVE0 --read-only
```

You can use the [firewall-rule.ps1](firewall-rule.ps1) script to add the needed firewall rules to
the windows firewall.