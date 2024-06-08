# donut.live

A tribute to [parrot.live](https://github.com/hugomd/parrot.live). Here it is in donut form.

## Demo - Try It
```
curl donut.rustychads.com
```

<div align="center" style="height: 40vh">
  <img src="https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExaWZvZm1kZ3dia2hjdXQwajU0eTBsM3g3NGJzMTdzMnJ2Y2hlZjJueSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/PCZgwB0fEhRzcbNH6Z/source.gif"/>
</div>

## Description
- HTTP server that distributes frames of a rotating donut to every [curl](https://en.wikipedia.org/wiki/CURL) client.

## Usage
```
Usage: donut-live [OPTIONS]

Options:
  -a, --addr <ADDR>  IP address [default: 127.0.0.1]
  -p, --port <PORT>  Port number [default: 8080]
      --path <PATH>  Location path [default: /]
  -h, --help         Print help
  -V, --version      Print version
```
## Todo
+ Improve [trim_frames](https://github.com/splurf/donut.live/blob/master/src/util.rs#L83) by removing **all** possible redundant ASCII-whitespace from every frame.

## Notes
+ This works for terminals that support [ANSI Escape Sequences](https://en.wikipedia.org/wiki/ANSI_escape_code)

## Credit
+ The [gen_frame](https://github.com/splurf/donut.live/blob/master/src/util.rs#L12) function within `utils.rs` heavily references the original [donut.c](https://www.a1k0n.net/2011/07/20/donut-math.html) script