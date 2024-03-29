let
  pkgs = import <nixpkgs> { };

  libraries = with pkgs;[
    webkitgtk_4_1
    gtk3
    cairo
    gdk-pixbuf
    glib
    dbus
    openssl_3
    libayatana-appindicator
    cargo
    rustc
    libsoup_3
  ];

  packages = with pkgs; [
    webkitgtk_4_1
    pkg-config
    dbus
    openssl_3
    glib
    gtk3
    libsoup
    webkitgtk
    appimagekit
    cargo
    rustc
    nodejs
    nodePackages.pnpm
  ];
in
pkgs.mkShell {
  buildInputs = packages;

  shellHook =
    ''
      export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
      export GIO_MODULE_DIR=${pkgs.glib-networking}/lib/gio/modules/
      export WEBKIT_DISABLE_COMPOSITING_MODE=1
    '';
}