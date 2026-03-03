# Operations: Building and Serving Docs

## Install mdBook

The repo uses a pinned `mdBook` binary installed into `.tools/`:

- Install: `./tools/install-mdbook.sh`
- Binary path: `.tools/mdbook/bin/mdbook`

## Build static output

Build the static HTML site:

- `make docs-build`

By default, mdBook writes static output into `docs/book/`.

## Local preview server

Run a local dev server:

- `make docs-serve`

This runs `mdbook serve` bound to `127.0.0.1:3000`.
