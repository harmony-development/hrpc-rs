{ release ? false
, doCheck ? false
, doDoc ? false
, common
,
}:
with common;
let
  meta = with pkgs.stdenv.lib; {
    description = "Implementation of hRPC in Rust.";
    homepage = "https://github.com/harmony-development/hrpc-rs";
    license = licenses.mit;
  };

  package = with pkgs; naersk.buildPackage {
    root = ../.;
    nativeBuildInputs = crateDeps.nativeBuildInputs;
    buildInputs = crateDeps.buildInputs;
    # WORKAROUND because doctests fail to compile (they compile with nightly cargo but then rustdoc fails)
    cargoTestOptions = def: def ++ [ "--lib" "--tests" "--bins" "--examples" ];
    override = (prev: env);
    overrideMain = (prev: {
      inherit meta;
    });

    inherit release doCheck doDoc;
  };
in
package
