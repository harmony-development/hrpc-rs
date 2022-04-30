{
  inputs.nci.url = "github:yusdacra/nix-cargo-integration";

  outputs = inputs:
    inputs.nci.lib.makeOutputs {
      root = ./.;
      overrides = {
        crateOverrides = common: prev: {
          interop = old: let
            env = {PROTOC = "protoc";};
          in
            {
              buildInputs = (old.buildInputs or []) ++ [common.pkgs.protobuf];
              propagatedEnv = env;
            }
            // env;
        };
      };
    };
}
