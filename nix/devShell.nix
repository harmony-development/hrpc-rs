{ common }:
with common; with pkgs;
devshell.mkShell {
  packages = [ rustc binutils ] ++ crateDeps.nativeBuildInputs ++ crateDeps.buildInputs;
  commands =
    let
      pkgCmd = pkg: { package = pkg; };
    in
    [
      (pkgCmd git)
      (pkgCmd nixpkgs-fmt)
      (pkgCmd cachix)
    ];
  env = [ ] ++ env;
}
