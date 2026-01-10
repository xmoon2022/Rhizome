{
  description = "Rhizome - 基于递归稳态迭代协议的自控管理工具";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];
        
        buildInputs = with pkgs; [
          # 用于终端支持
        ];
        
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rhizome";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit nativeBuildInputs buildInputs;
          
          meta = with pkgs.lib; {
            description = "Rhizome - 基于递归稳态迭代协议的自控管理工具";
            homepage = "https://github.com/xmoon/rhizome";
            license = licenses.gpl3Plus;
            maintainers = [];
            mainProgram = "rhizome";
          };
        };
        
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
        
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          
          shellHook = ''
            echo "🌳 Rhizome 开发环境"
            echo "运行 'cargo run' 启动程序"
          '';
        };
      }
    );
}
