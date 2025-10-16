# FilesLink CLI Usage

## Overview
The CLI tool helps manage permissions and control the bot.

## Usage
```bash
./fileslink-cli [OPTIONS] <SUBCOMMAND>
```

### Options
- `--path` Path to the FIFO (default: `/tmp/fileslink.pipe`, env: `FILESLINK_PIPE_PATH`)

### Subcommands
- `update-permissions` Updates permissions from the config file
- `shutdown` Shuts down the system
- `help` Prints help message

## Docker Example
```bash
docker exec -it fileslink-app fileslink-cli update-permissions
```
