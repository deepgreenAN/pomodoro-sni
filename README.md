# pomodoro-sni

A simple Pomodoro Timer for Linux displays on status bars as an implementation of [StatusNotifierItem](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/?__goaway_challenge=meta-refresh&__goaway_id=ff7c89a0ae4e647a4fa3e3f8ea2178ae&__goaway_referer=https%3A%2F%2Fwww.freedesktop.org%2F).

![screen shot of icon and contextmenu](screenshot_2.png)

## Features

- You can configure times, sound files, sound volume, colors, and long-break positions.  
- The pomodoro-sni implements StatusNotifierItem interface, so it works without any additional shell extensions or status bar implementations in many desktop environments.
- No dependencies on GUI toolkit. It depends on the dbusmenu protocol.
- You can start timer from external process or apps via `pomodoro-sni start` command. It is useful for scheduling the start time.

## Installation

### cargo

```
cargo install pomodoro-sni
```

### Prebuilt binary

You can download pre-built binary from the github [release page]().
