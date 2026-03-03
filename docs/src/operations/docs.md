# Docs Tooling

This repository uses mdBook for documentation.

```mermaid
flowchart LR
  Src[docs/src] --> Build[mdbook build]
  Build --> Out[docs/book (generated)]
```

## Build locally

```bash
./tools/install-mdbook.sh
./tools/install-mdbook-mermaid.sh
make docs-build
```

## Serve locally

```bash
make docs-serve
```

## Hygiene
Generated output under `docs/book/` must not be committed.

```bash
make docs-hygiene
```

