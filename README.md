# screensnap
A simple screenshooting program written in Rust.

Currently only supports Windows; PRs are welcome to add support for other
platforms.

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

### v1.1.0
- Improvement: smarter window filters (can now crop screenshots to more kinds of
    windows, e.g. [Telegram Desktop](
    https://github.com/telegramdesktop/tdesktop)).
- Fix: better support for Windows 10's virtual desktops
- Fix: better support for multiple monitor setups
- Change: window screenshots now contain the non-client area
- Internal: Refactor the `screengrab` module
- Internal: Use the `num-traits` crate

### v1.0.0
- Initial release
