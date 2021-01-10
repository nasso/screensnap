# screensnap
A simple screenshooting program written in Rust.

Currently only supports Windows; PRs are welcome to add support for other
platforms.

## IF YOU'RE ON WINDOWS 10 YOU PROBABLY DON'T NEED THIS.

Try `WINDOWS KEY + SHIFT + S`.

## Usage

Just run the executable. As of writing, no command line arguments are needed.

While the process is running, it waits for you to press the `Print Screen` key.
It doesn't do anything else (besides setting up the window and OpenGL context so
that they're ready as soon as you press the `Print Screen` key, but that only
happens once at startup).

When you press `Print Screen`, your screen will darken. This is the `screensnap`
cropping window. You can select a rectangular area by clicking and dragging with
the left click of the mouse. Alternatively, holding `shift` will allow you to
crop the screenshot to a window of your choice.

As soon as you release, the cropping window closes, and the area you've selected
gets copied to your clipboard. Most softwares support pasting images directly
from the clipboard.

## Key bindings and keystrokes

### System-wide

Key/keystroke                 | Action
----------------------------- | ---------------------------------------------
`Print Screen/SysRq/Snapshot` | Take a screenshot (opens the cropping window)

### In the cropping window

The cropping window is the window that opens when you press `Print Screen`. It
darkens the screen and lets you select a rectangle you want to copy to the
clipboard. It has a few handy keystrokes:

Key/keystroke  | Action
-------------- | ---------------------------------------------
`Shift` (hold) | Crop screenshot to individual windows
`Ctrl-Shift-Q` | Kill the process (disables system-wide keystrokes)

## Changelog

### next release... (`master` branch)
- feat: support for higher DPI settings

### v1.1.0
- feat: smarter window filters (can now crop to more kinds of windows, e.g.
    [Telegram Desktop](https://github.com/telegramdesktop/tdesktop)).
- feat: window screenshots now contain the non-client area
- fix: windows in other virtual desktops being "visible" to screensnap
- fix: weird behavior when the main monitor isn't the upper-left one

### v1.0.0
- Initial release
