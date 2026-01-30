{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixpkgs-legacy.url = "github:nixos/nixpkgs/nixos-23.11";
  };

  outputs = { self, nixpkgs, nixpkgs-legacy, ...}:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      pkgs-legacy = nixpkgs-legacy.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustup
          espup
          espflash
          ldproxy
          libz
          stdenv.cc.cc.lib
          pkgs-legacy.libxml2.out
          zlib
        ];

        env = {
            "LD_LIBRARY_PATH" = "${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib:${pkgs-legacy.libxml2.out}/lib:$LD_LIBRARY_PATH";
            "PATH" = "~/.rustup/toolchains/esp/xtensa-esp-elf/esp-13.2.0_20240530/xtensa-esp-elf/bin:~/.cargo/bin:$PATH";
        };

        shellHook = ''
          if [ ! -d  ~/.rustup/toolchains/esp ]; then espup install; fi;
          #export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib:${pkgs-legacy.libxml2.out}/lib:$LD_LIBRARY_PATH"
          #export PATH="~/.rustup/toolchains/esp/xtensa-esp-elf/esp-13.2.0_20240530/xtensa-esp-elf/bin:~/.cargo/bin:$PATH"
        '';
      };
    };
}