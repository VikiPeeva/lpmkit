# LPM Kit

Local process models: discovery, occurrence alignments, and fingerprints.
Built on [rust4pm](https://crates.io/crates/process_mining) for event logs and Petri nets.

## Status

Early skeleton. The only functionality so far is reading local process models
from a zip archive of PNML files (`core::io::read_lpm_zip`).


## Layout

```
src/
  lib.rs        crate root
  core/
    mod.rs      core module
    lpm.rs      Lpm: a Petri net and its name
    io.rs       read_lpm_zip: one Lpm per *.pnml in a zip archive
```

## Dependencies

- `process_mining` (rust4pm): event logs, Petri nets, PNML/XES import
- `zip`: reading model archives
- `thiserror`: error types

Requires Rust 1.88+ (rust4pm's MSRV).