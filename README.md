# capitu
My personal screen capture assistant for Sway.

If you intend to use this, drop an issue and I'll try to make it more presentable.


## Installing
```shell
~ $ cargo install --git https://github.com/cauebs/capítu
```


## Usage
```shell
~ $ capitu --help
...

USAGE:
    capitu [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help         Prints help information
    -s, --selection    Selects a region or window to be captured
    -V, --version      Prints version information

SUBCOMMANDS:
    help          Prints this message or the help of the given subcommand(s)
    kill          Stops recording by killing all wf-recorder processes
    record        Starts a video recording
    screenshot    Takes a screenshot
```
```shell
~ $ capitu record --help
...

FLAGS:
    -a, --audio      Captures audio when recording video
...
```
```shell
~ $ capitu screenshot --help
...

FLAGS:
    -c, --copy       Copies to clipboard instead of saving to a file
...
```
