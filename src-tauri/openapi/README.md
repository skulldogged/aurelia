# Jellyfin OpenAPI Specification

This directory contains the Jellyfin OpenAPI specification used to generate the API client.

## Current Version
- **jellyfin-openapi-10.11.json** - Jellyfin 10.11.2 (stable)

## Updating the Specification

To update to a newer version:

1. Find the latest stable spec at: https://repo.jellyfin.org/files/openapi/jellyfin-openapi-stable.json
2. Download it:
   ```bash
   curl -L https://repo.jellyfin.org/files/openapi/jellyfin-openapi-stable.json -o openapi/jellyfin-openapi-10.XX.json
   ```
3. Update `build.rs` to reference the new file name
4. Run `cargo build` to regenerate the client

## Generated Code

The OpenAPI spec is processed at build time by `progenitor` in `build.rs`, which generates:
- API endpoint methods (ItemsApi, UserApi, PlaystateApi, etc.)
- Request/response models (BaseItemDto, UserDto, etc.)

The generated code is placed in `src/api/generated/` and should not be edited manually.
