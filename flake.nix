{
  description = "BigLinux WebApps — turn websites into desktop apps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Runtime libraries the three binaries link against.
        runtimeDeps = with pkgs; [
          glib
          gtk4
          libadwaita
          webkitgtk_6_0
          openssl
          gettext
        ];

        # Tools needed only during the build.
        buildDeps = with pkgs; [
          pkg-config
          wrapGAppsHook4
          gettext # msgfmt
        ];
      in {
        packages = rec {
          default = biglinux-webapps;

          biglinux-webapps = pkgs.rustPlatform.buildRustPackage {
            pname = "biglinux-webapps";
            version = "4.0.0";

            src = pkgs.lib.cleanSource ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = buildDeps;
            buildInputs = runtimeDeps;

            # Compile message catalogs before install.
            preBuild = ''
              for po in po/*.po; do
                lang=$(basename "$po" .po)
                msgfmt -o "po/''${lang}.mo" "$po"
              done
            '';

            # Install data files after cargo installs the binaries.
            postInstall = ''
              install -Dm644 biglinux-webapps/usr/share/applications/br.com.biglinux.webapps.desktop \
                $out/share/applications/br.com.biglinux.webapps.desktop

              install -Dm644 biglinux-webapps/usr/share/metainfo/br.com.biglinux.webapps.metainfo.xml \
                $out/share/metainfo/br.com.biglinux.webapps.metainfo.xml

              install -Dm644 biglinux-webapps/usr/share/icons/hicolor/scalable/apps/big-webapps.svg \
                $out/share/icons/hicolor/scalable/apps/big-webapps.svg
              install -Dm644 biglinux-webapps/usr/share/icons/hicolor/scalable/apps/big-webapps-symbolic.svg \
                $out/share/icons/hicolor/scalable/apps/big-webapps-symbolic.svg

              install -Dm644 biglinux-webapps/usr/share/biglinux-webapps/browsers.toml \
                $out/share/biglinux-webapps/browsers.toml

              for mo in po/*.mo; do
                lang=$(basename "$mo" .mo)
                locale_dir=''${lang//-/_}
                install -Dm644 "$mo" \
                  "$out/share/locale/''${locale_dir}/LC_MESSAGES/biglinux-webapps.mo"
              done
            '';

            meta = with pkgs.lib; {
              description = "Turn any website into a desktop app (GTK4/libadwaita)";
              homepage = "https://github.com/biglinux/biglinux-webapps";
              license = licenses.gpl3Plus;
              platforms = platforms.linux;
              mainProgram = "big-webapps-gui";
            };
          };
        };

        # `nix run .` launches the GUI.
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.biglinux-webapps}/bin/big-webapps-gui";
        };

        # `nix develop` drops into a shell with cargo + all native deps wired up.
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.biglinux-webapps ];
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            cargo-outdated
            cargo-audit
          ];
          shellHook = ''
            echo "biglinux-webapps dev shell — run: cargo build --release --workspace"
          '';
        };
      });
}
