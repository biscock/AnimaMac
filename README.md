# AnimaTux
This is my recreation of Animaengine in Rust for linux, though I don't see why you couldn't use it on Windows.

## Features
 - play animations or show images on your desktop or whatever app you are using
 - Supports original Animaengine workshop items (requires steamcmd and ffmpeg on your system)
 - Works on most desktops (Wayland and X11)

## Wayland
Wayland doesn't allow apps to show above others, however the user can normally set this by right clicking the title bar and selecting Always On Top in KDE or GNOME (and likely others)

If you use X11 you *should* be okay by default.
