{
  description = "SvelteKit development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc

              # Rust libs
              pkg-config
              alsa-lib

              (vscode-with-extensions.override {
                vscode = vscodium;
                vscodeExtensions = with vscode-extensions; [
                  dbaeumer.vscode-eslint
                  prettier.prettier-vscode
                  svelte.svelte-vscode
                  rust-lang.rust-analyzer
                ] ++ pkgs.vscode-utils.extensionsFromVscodeMarketplace [
                  {
                    name = "vscode-teal";
                    publisher = "pdesaulniers";
                    version = "0.11.0";
                    sha256 = "sha256-idIXnJ5NedwYwuEBPjKc7+UWztodpFboSSfp+oFFNic=";
                  }
                ];
              })
            ];
          };
        }
      );
    };
}
