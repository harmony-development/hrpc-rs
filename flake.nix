{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      overrides = {
        crateOverrides = common: prev: {
          interop = prevv: let
            env = {PROTOC = "protoc";};
          in
            {
              buildInputs = (prevv.buildInputs or []) ++ [common.pkgs.protobuf];
              propagatedEnv = env;
            }
            // env;
        };
      };
    };
}
