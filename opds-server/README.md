# livtet-opds-server

OPDS 1.x and 2.0 HTTP server, embedded in the desktop Tauri app. Exposes the local library as a browsable, searchable catalog to any OPDS-aware client.

## What It Is

The piece of Livtet that turns the local SQLite library into a standard OPDS endpoint. It serves root + navigation feeds, search, paginated publication lists, and acquisition downloads directly from the database, gated by the user's OPDS-on/off preference and password authentication. The Tauri app embeds it on a configurable port; mobile clients and third-party reading apps consume it through the same `livtet-opds-client` crate the desktop uses.

## Build & Test

```bash
mise run test-rust -p livtet-opds-server
```
