## About

lxc-tool is tool to download with LXC images files.

## Build

```bash
cargo build --release
```

## Usage

### Configuration file

Currently, the configuration file is minimalistic and self-documented.

### Command line options

At the moment, only one option "download-images" is implemented.

```bash
lxc-tool download-images
```


## Log to console

The tool can write a log to STDOUT instead of syslog. Just define the environment variable RUST_LOG with the desired log level:

```bash
RUST_LOG=info lxc-tool download-images ...
```

