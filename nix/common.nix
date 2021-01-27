{ sources, system }:
let
  pkgz = import sources.nixpkgs { inherit system; overlays = [ sources.rustOverlay.overlay ]; };
  rustChannel = pkgz.rust-bin."stable".latest;

  pkgs = import sources.nixpkgs {
    inherit system;
    overlays = [
      sources.rustOverlay.overlay
      sources.devshell.overlay
      (final: prev: {
        rustc = rustChannel.rust.override {
          extensions = [ "rust-src" ];
        };
      })
      (final: prev: {
        naersk = prev.callPackage sources.naersk { };
      })
    ];
  };
in
{
  inherit pkgs;

  /* You might need this if your application utilizes a GUI. Note that the dependencies
    might change from application to application. The example dependencies provided here
    are for a typical iced application that uses Vulkan underneath.

    For example, it might look like this:

    neededLibs = with pkgs; (with xorg; [ libX11 libXcursor libXrandr libXi ])
    ++ [ vulkan-loader wayland wayland-protocols libxkbcommon ];
  */
  neededLibs = [ ];

  # Dependencies listed here will be passed to Nix build and development shell
  crateDeps =
    with pkgs;
    {
      buildInputs = [ protobuf ];
      nativeBuildInputs = [ /* Add compile time dependencies here */ ];
    };

  /* Put env variables here, like so:

    env = {
      PROTOC = "${pkgs.protobuf}/bin/protoc";
    };

    The variables are not (shell) escaped.
    Variables put here will appear in both dev env and build env.
  */
  env = {
    PROTOC = "${pkgs.protobuf}/bin/protoc";
    PROTOC_INCLUDE = "${pkgs.protobuf}/include";
  };
}
