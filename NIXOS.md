# NixOS Module for Aurelia Sidecar Lyrics Daemon

This document describes how to use the NixOS module for the Aurelia Sidecar Lyrics Daemon.

## Quick Start

Add Aurelia to your flake inputs:

```nix
{
  inputs.aurelia.url = "github:aurelia-music/aurelia";
  
  outputs = { self, nixpkgs, aurelia, ... }: {
    nixosConfigurations.myserver = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        aurelia.nixosModules.aurelia-sidecar-daemon
        {
          services.aurelia-sidecar-daemon = {
            enable = true;
            settings = {
              jellyfin_url = "http://localhost:8096";
              # Don't put API keys here in production!
              # Use environmentFile instead
              music_paths = [ "/var/lib/jellyfin/media/music" ];
              bind = "0.0.0.0";
              port = 8080;
            };
            environmentFile = "/var/secrets/aurelia-sidecar.env";
          };
        }
      ];
    };
  };
}
```

## Configuration Options

### Basic Options

- `enable`: Enable the service
- `package`: The package to use (defaults to the one from the flake)
- `openFirewall`: Open the firewall for the daemon port
- `environmentFile`: Path to a file containing environment variables (for secrets)

### Settings

All settings are under the `settings` attribute:

- `jellyfin_url`: URL of your Jellyfin server (e.g., "http://localhost:8096")
- `jellyfin_api_key`: API key for Jellyfin (use environmentFile instead!)
- `music_paths`: List of paths to scan for sidecar files
- `bind`: Address to bind to (default: "127.0.0.1")
- `port`: Port to listen on (default: 8080)
- `cache_ttl_seconds`: How long to cache lyrics (default: 3600)

## Security Best Practices

### Using an Environment File for Secrets

Create a secrets file that is not in your git repository:

```bash
# /var/secrets/aurelia-sidecar.env
JELLYFIN_API_KEY=your-secret-api-key-here
```

Set proper permissions:

```bash
sudo chmod 600 /var/secrets/aurelia-sidecar.env
sudo chown root:root /var/secrets/aurelia-sidecar.env
```

Then reference it in your configuration:

```nix
services.aurelia-sidecar-daemon = {
  enable = true;
  settings = {
    jellyfin_url = "http://localhost:8096";
    music_paths = [ "/var/lib/jellyfin/media" ];
  };
  environmentFile = "/var/secrets/aurelia-sidecar.env";
};
```

### Running with Jellyfin

The service automatically adds a dependency on Jellyfin if you configure a Jellyfin URL:

```nix
services.aurelia-sidecar-daemon = {
  enable = true;
  settings.jellyfin_url = "http://localhost:8096";
  # This will add jellyfin.service to the After= dependency
};
```

## Complete Example

Here's a complete NixOS configuration:

```nix
{ config, pkgs, ... }:

{
  # Enable Jellyfin
  services.jellyfin = {
    enable = true;
    openFirewall = true;
  };

  # Enable the lyrics daemon
  services.aurelia-sidecar-daemon = {
    enable = true;
    settings = {
      jellyfin_url = "http://localhost:8096";
      music_paths = [ "/var/lib/jellyfin/media/music" ];
      bind = "127.0.0.1";
      port = 8080;
      cache_ttl_seconds = 3600;
    };
    environmentFile = config.sops.secrets.aurelia-sidecar-env.path; # Using sops-nix
  };

  # Make it accessible from the network (optional)
  services.nginx = {
    enable = true;
    recommendedProxySettings = true;
    
    virtualHosts."lyrics.myserver.com" = {
      enableACME = true;
      forceSSL = true;
      locations."/" = {
        proxyPass = "http://127.0.0.1:8080";
      };
    };
  };
}
```

## Building Manually

You can also build the package directly:

```bash
# Build the package
nix build github:aurelia-music/aurelia#packages.x86_64-linux.aurelia-sidecar-daemon

# Or with your local checkout
nix build .#aurelia-sidecar-daemon

# Run it
./result/bin/aurelia-sidecar-daemon --help
```

## Troubleshooting

### Check service status

```bash
systemctl status aurelia-sidecar-daemon
```

### View logs

```bash
journalctl -u aurelia-sidecar-daemon -f
```

### Test the API

```bash
# Health check
curl http://localhost:8080/health

# Get lyrics for a song (replace with your Jellyfin item ID)
curl http://localhost:8080/lyrics/YOUR-ITEM-ID-HERE
```

### Permission Issues

The service runs as a dynamic user and needs read access to your music directories:

```nix
services.aurelia-sidecar-daemon.settings.music_paths = [ 
  "/var/lib/jellyfin/media"  # Make sure this is readable
];

# Or add ACLs
systemd.tmpfiles.rules = [
  "a+ /var/lib/jellyfin/media - - - - +r"
];
```

## Using with sops-nix

If you're using [sops-nix](https://github.com/Mic92/sops-nix) for secret management:

```nix
{
  sops.secrets.aurelia-sidecar-env = {
    sopsFile = ./secrets.yaml;
    format = "yaml";
  };

  services.aurelia-sidecar-daemon = {
    enable = true;
    settings = {
      jellyfin_url = "http://localhost:8096";
      music_paths = [ "/var/lib/jellyfin/media" ];
    };
    environmentFile = config.sops.secrets.aurelia-sidecar-env.path;
  };
}
```

Your `secrets.yaml`:

```yaml
aurelia-sidecar-env: |
  JELLYFIN_API_KEY=super-secret-key
```
