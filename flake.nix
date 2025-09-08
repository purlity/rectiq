{
  description = "Rectiq CLI";
  outputs = { self, nixpkgs }: let
    systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin" ];
    forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
  in {
    packages = forAll (pkgs: let
      ver = builtins.getEnv "RECTIQ_VERSION";
      urlBase = tag: "https://github.com/purlity/rectiq/releases/download/${tag}";
      isDarwin = pkgs.stdenv.hostPlatform.isDarwin;
      isAarch64 = pkgs.stdenv.hostPlatform.isAarch64;
    in {
      rectiq = pkgs.stdenv.mkDerivation {
        pname = "rectiq";
        version = if ver == "" then "0.1.0" else ver;
        src = pkgs.fetchurl {
          url = "${urlBase "rectiq-cli-v${if ver == "" then "0.1.0" else ver}"}/" +
            (if isDarwin && isAarch64 then "rectiq-cli-${if ver == "" then "0.1.0" else ver}-aarch64-apple-darwin.tar.gz"
             else if isDarwin then "rectiq-cli-${if ver == "" then "0.1.0" else ver}-x86_64-apple-darwin.tar.gz"
             else "rectiq-cli-${if ver == "" then "0.1.0" else ver}-x86_64-unknown-linux-musl.tar.gz");
          # Placeholder; CI pins this for Linux (and can be extended for Darwin)
          sha256 = "0000000000000000000000000000000000000000000000000000"; # replaced in CI
        };
        unpackPhase = "tar -xzf $src";
        installPhase = ''
          mkdir -p $out/bin
          cp rectiq $out/bin/rectiq
        '';
      };
    });
    defaultPackage.x86_64-linux = self.packages.x86_64-linux.rectiq;
    defaultPackage.aarch64-darwin = self.packages.aarch64-darwin.rectiq;
    defaultPackage.x86_64-darwin = self.packages.x86_64-darwin.rectiq;
  };
}

