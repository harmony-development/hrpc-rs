{ common }:
with common; with pkgs;
devshell.mkShell {
  packages =
    [ git nixpkgs-fmt rustc binutils ]
    ++ crateDeps.nativeBuildInputs ++ crateDeps.buildInputs;
  env = {
    LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${lib.makeLibraryPath neededLibs}";
  } // env;
}
