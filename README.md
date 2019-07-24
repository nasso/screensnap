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
