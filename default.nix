with import <nixpkgs> {};
let dependencies = [
  stdenv
  glib
  cairo
  pkg-config
  gnome2.pango
  gdk_pixbuf
  atk
  gnome3.gtk
  xdotool
];
in {
  env = stdenv.mkDerivation {
    name = "emoji-quickpick-env";
    buildInputs = dependencies;
    CPATH = "${stdenv.cc.libc.dev}/include";
    LD_LIBRARY_PATH = stdenv.lib.makeLibraryPath dependencies;
  };
}
