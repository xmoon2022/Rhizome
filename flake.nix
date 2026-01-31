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
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        nativeBuildInputs = with pkgs; [ rustToolchain pkg-config ];

        buildInputs = with pkgs;
          [
            # 用于终端支持
          ];

        # 定义一个过滤器，排除 target 和 .git 目录
        srcFilter = path: type:
          let baseName = baseNameOf path;
          in baseName != "target" && baseName != ".git" && baseName != "result";

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rhizome";
          version = "0.1.0";

          # 1. 优化源代码输入：只包含源码，剔除 target 和 .git
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = srcFilter;
          };

          cargoLock = { lockFile = ./Cargo.lock; };

          inherit nativeBuildInputs buildInputs;

          # 2. 强制本地构建：告诉 Nix 不要把这个任务发给远程机器
          preferLocalBuild = true;

          meta = with pkgs.lib; {
            description = "Rhizome - 基于递归稳态迭代协议的自控管理工具";
            homepage = "https://github.com/xmoon2022/rhizome";
            license = licenses.gpl3Plus;
            maintainers = [ ];
            mainProgram = "rhizome";
          };
        };

        apps.default =
          flake-utils.lib.mkApp { drv = self.packages.${system}.default; };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "🌳 Rhizome 开发环境"
            echo "运行 'cargo run' 启动程序"
          '';
        };
      });
}
