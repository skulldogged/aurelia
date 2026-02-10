{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        inherit (nixpkgs) lib;

        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system;
          inherit overlays;
          config.allowUnfree = true;
          config.android_sdk.accept_license = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
          targets =
            lib.optionals pkgs.stdenv.isDarwin [
              "aarch64-apple-ios"
              "aarch64-apple-ios-sim"
            ]
            ++ [
              "aarch64-linux-android"
              "armv7-linux-androideabi"
              "i686-linux-android"
              "x86_64-linux-android"
            ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common source filtering for the Rust workspace
        src = let
          # Include Rust source, Cargo files, test fixtures, and the quick-xml
          # templates that are needed at build time.
          sourceFilter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*\\.ttml$" path != null)
            || (builtins.match ".*\\.toml$" path != null);
        in
          lib.cleanSourceWith {
            src = ./.;
            filter = sourceFilter;
          };

        # Common Cargo arguments shared between deps and the final build
        commonArgs = {
          inherit src;
          pname = "aurelia-web-backend";
          version = "0.1.0";
          strictDeps = true;
          # Only build the web backend binary
          cargoExtraArgs = "-p aurelia-web-backend";
        };

        # Build just the cargo dependencies for caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the web backend binary
        aurelia-web-backend = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            # Skip tests during package build (run them separately)
            doCheck = false;
          });

        # Build the frontend with bun
        aurelia-web-frontend = pkgs.stdenv.mkDerivation {
          pname = "aurelia-web-frontend";
          version = "0.1.0";

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type: let
              relPath = lib.removePrefix (toString ./.) (toString path);
            in
              # Include frontend, shared, and root config files
              lib.hasPrefix "/apps/web/frontend" relPath
              || lib.hasPrefix "/apps/shared" relPath
              || relPath == "/package.json"
              || relPath == "/bun.lock"
              || relPath == "/tsconfig.json"
              || (type == "directory");
          };

          nativeBuildInputs = [pkgs.bun pkgs.cacert];

          # Bun needs a writable home directory
          HOME = "/tmp/bun-home";

          buildPhase = ''
            runHook preBuild
            mkdir -p $HOME

            # Install dependencies from the workspace root
            bun install --frozen-lockfile

            # Build the frontend
            cd apps/web/frontend
            bun run build

            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            cp -r dist $out
            runHook postInstall
          '';
        };

        # Combined package: backend binary + frontend static files
        aurelia-web = pkgs.symlinkJoin {
          name = "aurelia-web";
          version = "0.1.0";
          paths = [aurelia-web-backend];
          postBuild = ''
            # Create a share directory with the frontend files
            mkdir -p $out/share/aurelia-web
            cp -r ${aurelia-web-frontend}/* $out/share/aurelia-web/
          '';
        };

        # Build the sidecar lyrics daemon
        commonArgsSidecar = {
          inherit src;
          pname = "aurelia-sidecar-daemon";
          version = "0.1.0";
          strictDeps = true;
          cargoExtraArgs = "-p aurelia-sidecar-daemon";
        };

        cargoArtifactsSidecar = craneLib.buildDepsOnly commonArgsSidecar;

        aurelia-sidecar-daemon = craneLib.buildPackage (commonArgsSidecar
          // {
            inherit cargoArtifactsSidecar;
            doCheck = false;
          });

        androidComposition = pkgs.androidenv.composeAndroidPackages {
          cmdLineToolsVersion = "13.0";
          platformToolsVersion = "36.0.1";
          buildToolsVersions = ["36.0.0"];
          platformVersions = ["36"];
          abiVersions = ["arm64-v8a" "x86_64"];
          includeNDK = true;
          ndkVersions = ["29.0.14206865"];
        };

        androidSdk = androidComposition.androidsdk;
      in {
        packages = {
          inherit aurelia-web-backend aurelia-web-frontend aurelia-web aurelia-sidecar-daemon;
          default = aurelia-web;
        };

        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs;
            [
              pkg-config
              gobject-introspection
              cargo-tauri
              rustToolchain
              bun
              wrapGAppsHook4
              jdk17
              cargo-ndk
            ]
            ++ lib.optionals stdenv.isLinux [
              android-studio
            ];

          buildInputs = with pkgs;
            lib.optionals stdenv.isLinux [
              alsa-lib
              at-spi2-atk
              atkmm
              bitwarden-cli
              cairo
              gdk-pixbuf
              glib
              gtk3
              harfbuzz
              librsvg
              libsoup_3
              pango
              webkitgtk_4_1
              openssl
              libayatana-appindicator
            ];

          ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
          NDK_HOME = "${androidSdk}/libexec/android-sdk/ndk/29.0.14206865";
          JAVA_HOME = pkgs.jdk17.home;
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          __NV_DISABLE_EXPLICIT_SYNC = 1;
          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
            export PATH="${androidSdk}/libexec/android-sdk/platform-tools:$PATH"
            ${lib.optionalString pkgs.stdenv.isDarwin "unset SDKROOT"}
          '';
        };
      }
    )
    // {
      # NixOS module (system-independent)
      nixosModules = {
        default = _: {
          imports = [
            self.nixosModules.aurelia-web
            self.nixosModules.aurelia-sidecar-daemon
          ];
        };

        aurelia-web = {
          config,
          lib,
          pkgs,
          ...
        }: let
          cfg = config.services.aurelia;
        in {
          options.services.aurelia = {
            enable = lib.mkEnableOption "Aurelia web music player";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.aurelia-web;
              description = "The Aurelia web package to use.";
            };

            host = lib.mkOption {
              type = lib.types.str;
              default = "0.0.0.0";
              description = "Address to bind the server to.";
            };

            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
              description = "Port to listen on.";
            };

            dataDir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/aurelia";
              description = "Directory for Aurelia's persistent data (database, cache).";
            };

            user = lib.mkOption {
              type = lib.types.str;
              default = "aurelia";
              description = "User account under which Aurelia runs.";
            };

            group = lib.mkOption {
              type = lib.types.str;
              default = "aurelia";
              description = "Group under which Aurelia runs.";
            };

            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Whether to open the firewall port.";
            };

            environment = lib.mkOption {
              type = lib.types.attrsOf lib.types.str;
              default = {};
              description = "Extra environment variables for the Aurelia service.";
              example = lib.literalExpression ''
                {
                  RUST_LOG = "info";
                }
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            users.users.${cfg.user} = {
              inherit (cfg) group;

              isSystemUser = true;
              home = cfg.dataDir;
              createHome = true;
            };

            users.groups.${cfg.group} = {};

            networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [cfg.port];

            systemd.services.aurelia = {
              description = "Aurelia Web Music Player";
              after = ["network.target"];
              wantedBy = ["multi-user.target"];

              environment =
                {
                  AURELIA_HOST = cfg.host;
                  AURELIA_PORT = toString cfg.port;
                  AURELIA_DATA_DIR = cfg.dataDir;
                  AURELIA_STATIC_DIR = "${cfg.package}/share/aurelia-web";
                  RUST_LOG = "info";
                }
                // cfg.environment;

              serviceConfig = {
                Type = "simple";
                User = cfg.user;
                Group = cfg.group;
                ExecStart = "${cfg.package}/bin/aurelia-web-backend";
                Restart = "on-failure";
                RestartSec = 5;

                # Hardening
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = [cfg.dataDir];
                PrivateTmp = true;
                PrivateDevices = true;
                ProtectKernelTunables = true;
                ProtectControlGroups = true;
                RestrictSUIDSGID = true;
              };
            };
          };
        };

        # NixOS module for the sidecar lyrics daemon
        aurelia-sidecar-daemon = {
          config,
          lib,
          pkgs,
          ...
        }: let
          cfg = config.services.aurelia-sidecar-daemon;
          settingsFormat = pkgs.formats.toml {};
          # Filter out null values since TOML doesn't support them
          settingsFiltered = lib.filterAttrs (n: v: v != null) cfg.settings;
          configFile = settingsFormat.generate "aurelia-sidecar-daemon.toml" settingsFiltered;
        in {
          options.services.aurelia-sidecar-daemon = {
            enable = lib.mkEnableOption "Aurelia Sidecar Lyrics Daemon";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.aurelia-sidecar-daemon;
              description = "The aurelia-sidecar-daemon package to use.";
            };

            settings = lib.mkOption {
              type = lib.types.submodule {
                freeformType = settingsFormat.type;
                options = {
                  jellyfin_url = lib.mkOption {
                    type = lib.types.nullOr lib.types.str;
                    default = null;
                    description = "URL of the Jellyfin server.";
                    example = "http://localhost:8096";
                  };

                  jellyfin_api_key = lib.mkOption {
                    type = lib.types.nullOr lib.types.str;
                    default = null;
                    description = ''
                      Jellyfin API key for authentication.
                      WARNING: Use environmentFile option instead for production deployments.
                    '';
                  };

                  music_paths = lib.mkOption {
                    type = lib.types.listOf lib.types.path;
                    default = [];
                    description = "Paths to music libraries to scan for sidecar files.";
                    example = ["/var/lib/jellyfin/media/music"];
                  };

                  bind = lib.mkOption {
                    type = lib.types.str;
                    default = "127.0.0.1";
                    description = "Address to bind the HTTP server to.";
                  };

                  port = lib.mkOption {
                    type = lib.types.port;
                    default = 8080;
                    description = "Port to listen on.";
                  };

                  cache_ttl_seconds = lib.mkOption {
                    type = lib.types.ints.positive;
                    default = 3600;
                    description = "Time-to-live for cached lyrics in seconds.";
                  };
                };
              };
              default = {};
              description = "Configuration for the Aurelia Sidecar Lyrics Daemon.";
            };

            environmentFile = lib.mkOption {
              type = lib.types.nullOr lib.types.path;
              default = null;
              description = ''
                Path to an environment file containing sensitive configuration.
                Useful for storing the Jellyfin API key securely.
              '';
            };

            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = ''
                Whether to open the firewall for the daemon port.
                Only enable this if the daemon needs to be accessible from other machines.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.aurelia-sidecar-daemon = {
              description = "Aurelia Sidecar Lyrics Daemon";
              after = ["network.target"] ++ lib.optional (cfg.settings.jellyfin_url != null) "jellyfin.service";
              wantedBy = ["multi-user.target"];

              serviceConfig = {
                Type = "simple";
                DynamicUser = true;
                ExecStart = "${cfg.package}/bin/aurelia-sidecar-daemon --config ${configFile}";
                Restart = "on-failure";
                RestartSec = 5;
                StateDirectory = "aurelia-sidecar";

                # Security hardening
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = ["/var/lib/aurelia-sidecar"];
                PrivateTmp = true;
                PrivateDevices = true;
                ProtectKernelTunables = true;
                ProtectControlGroups = true;
                RestrictSUIDSGID = true;

                # Read music directories
                BindReadOnlyPaths = cfg.settings.music_paths;

                # Environment file for secrets
                EnvironmentFile = lib.mkIf (cfg.environmentFile != null) cfg.environmentFile;

                # Resource limits
                MemoryMax = "512M";
                CPUQuota = "50%";
              };
            };

            networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [cfg.settings.port];
          };
        };
      };
    };
}
