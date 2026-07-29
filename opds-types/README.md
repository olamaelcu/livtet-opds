# livtet-opds-types

Pure data types for the OPDS protocol layer. Shared between the client, the server, and the FFI surface that mobile apps consume.

## What It Is

The OPDS-specific vocabulary: catalogs, navigation feeds, publication entries, acquisition links, facet definitions, the JSON/Atom envelope types. The crate is intentionally I/O-free and format-agnostic so both the JSON-based client/server paths and the Atom-encoded legacy path can map onto the same model. Any consumer that needs to speak OPDS — server, client, mobile, third-party — pulls in these types.

## Build & Test

```bash
mise run test-rust -p livtet-opds-types
```

## Architecture

- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — workspace overview
- [docs/reference/opds/architecture.md](../../docs/reference/opds/architecture.md) — OPDS crate graph
- [docs/reference/opds/type-reference.md](../../docs/reference/opds/type-reference.md) — type catalog
- [docs/reference/adr/0009-opds-2-0-protocol.md](../../docs/reference/adr/0009-opds-2-0-protocol.md) — OPDS 2.0 decision
