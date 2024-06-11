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
HTTP server that distributes frames of a rotating donut to every [curl](https://en.wikipedia.org/wiki/CURL) client.
- The program can also be provided a custom GIF file (via the `--gif` flag). In this case, each frame from the GIF will be automatically converted into ASCII art. These frames will then be distributed with a frame rate based on the original delay of each frame. If the delay of each frame isn't properly set, then a manual frame rate is required. This can be fixed with the `--fps` flag.

## Usage
```
Usage: donut-live [OPTIONS]

Options:
  -a, --addr <ADDR>  IP address [default: 127.0.0.1]      
  -p, --port <PORT>  Port number [default: 8080]
      --path <PATH>  URI location path [default: /]       
  -g, --gif <GIF>    Custom provided GIF
  -f, --fps <FPS>    Custom Frames/sec
  -c, --colored      Enable/Disable color
  -h, --help         Print help
  -V, --version      Print version
```

## Other Live Demos
- `bad-apple.rustychads.com`
- `shrek.rustychads.com`

## Todo
+ Improve [trim_frames](https://github.com/splurf/donut.live/blob/4f13c1280d9ead28b1fb40d0d3f0d52429487958/src/base/donut.rs#L84) by removing **all** possible redundant ASCII-whitespace from every frame.

## Notes
+ This works for terminals that support [ANSI Escape Sequences](https://en.wikipedia.org/wiki/ANSI_escape_code).

## Credit
+ The [gen_frame](https://github.com/splurf/donut.live/blob/4f13c1280d9ead28b1fb40d0d3f0d52429487958/src/base/donut.rs#L13) function within `utils.rs` heavily references the original [donut.c](https://www.a1k0n.net/2011/07/20/donut-math.html) script.
