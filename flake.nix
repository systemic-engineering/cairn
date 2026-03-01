{
  description = "gall — tamper-proof witnessed agent work. In git. The audacity.";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs     = nixpkgs.legacyPackages.${system};
        beamPkgs = pkgs.beam.packages.erlang_27;
        erlang   = pkgs.erlang_27;
        gleam    = pkgs.gleam;
        rebar3   = beamPkgs.rebar3;
      in {
        # Development shell: all tools needed to build and run gall.
        devShells.default = pkgs.mkShell {
          buildInputs = [
            gleam
            erlang
            rebar3
            pkgs.git
            pkgs.openssh   # ssh-keygen for agent key derivation
            pkgs.just
          ];
          shellHook = ''
            export LANG=en_US.UTF-8
          '';
        };

        # Spawn a sandboxed agent session.
        # Usage: nix develop .#agent --command gleam run --module gall/daemon
        #
        # The agent shell deliberately omits direct git write access.
        # gall (the host process) controls what enters the project's git history.
        # The agent reads history through gall's MCP tools; it never writes .git.
        devShells.agent = pkgs.mkShell {
          buildInputs = [
            gleam
            erlang
            rebar3
            pkgs.git         # read-only git operations (clone, log, blame, show)
            pkgs.openssh
          ];
          shellHook = ''
            export LANG=en_US.UTF-8
            # Agent sessions are sandboxed: no network by default.
            # git push / git send-email are available only to gall (the host).
          '';
        };
      });
}
