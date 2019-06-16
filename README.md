# emoji-quickpick

`emoji-quickpick` is a tool that lets you quickly pick an emoji and insert it
into whereever you're typing.

## Usage

Configure your window manager to run `emoji-quickpick` whenever you press some
key combo (eg. I have it bound to Windows-E). When you run it you'll be
presented with a small text box in the middle of your screen. Start typing the
name of an emoji and a list of suggestions will appear. Press enter to use the
top suggestion, or use the arrow keys to scroll to another suggestion. Select
an emoji to close `emoji-quickpick` and enter the emoji into the focused
application (by simulating typing it). Or otherwise press escape to cancel.

## Building

`emoji-quickpick` is written in Rust so you'll need the Rust compiler as well
as the native dependencies: glib, cairo, pango, gdk_pixbuf, atk, gtk+ and
xdotool (for libxdo).

After you have these, just install through
`cargo`:

    cargo install emoji-quickpick

## Compatibility

Only tested on Linux and X windows. Unlikely to be compatible with anything
else.

## License

GPL-v2

