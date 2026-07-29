# livtet-opds-client

HTTP client for OPDS 1.x and 2.0 catalogs. Catalog discovery, search, pagination, and the download manager that fetches book files.

## What It Is

The piece of Livtet that talks to OPDS endpoints, whether a third-party reading-app catalog or a Livtet desktop's own OPDS server. It handles discovery (root + navigation feeds), search with facets, paginated publication lists, and the file-download lifecycle including resume, persistence, and integrity checks. Used by the desktop app's "Add catalog" flow, by the mobile clients through the FFI bridge, and by any plugin that wants to talk OPDS.

## Build & Test

```bash
mise run test-rust -p livtet-opds-client
```

## Architecture

- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — workspace overview
- [docs/reference/opds/architecture.md](../../docs/reference/opds/architecture.md) — OPDS crate graph
- [docs/reference/adr/0009-opds-2-0-protocol.md](../../docs/reference/adr/0009-opds-2-0-protocol.md) — OPDS 2.0 decision
- [docs/reference/adr/0024-opds-download-manager-persistence.md](../../docs/reference/adr/0024-opds-download-manager-persistence.md) — download persistence
