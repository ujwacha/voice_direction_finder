{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let

    pkgs = nixpkgs.legacyPackages."x86_64-linux";

    buildInputs = with pkgs; [
      rustPlatform.bindgenHook
      alsa-lib
      expat
      fontconfig
      freetype
      freetype.dev
      libGL
      pkg-config
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
      wayland
      libxkbcommon
    ];

  in {

    devShells."x86_64-linux".default = pkgs.mkShell {
      inherit buildInputs;
      packages = with pkgs; [
        cargo
        rustc
        clippy
        rustfmt
        rust-analyzer
        bacon
      ];
      nativeBuildInputs = [
        pkgs.pkg-config
      ];

      LD_LIBRARY_PATH =
        builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
