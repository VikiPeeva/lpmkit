# lpmkit

Local process models: discovery, occurrence alignments, and fingerprints.
Built on [rust4pm](https://crates.io/crates/process_mining) for event logs and Petri nets.

## Layout

```
src/
  core/         concepts  — Symbol, EventLog, Lpm, LpmSet          (+ io: zip of PNMLs, XES)
  align/        concept   — Occurrence, OccurrenceSet
                operation — align(), align_all()
                axes      — GapConstraint, Cost; engine sees the LPM via MoveModel
  discover/     operation — discover()
                axes      — SearchSpace (place.rs), QualityMeasure (measure.rs)
  fingerprint/  concept   — Fingerprint
                operation — fingerprint(); context extraction is private
```

Rules the layout follows:

- **Structs are concepts, traits are design-space axes, modules are chapters.**
- A module only depends on modules above it: `core` → `align` → `discover` → `fingerprint`.
- Cross-module values hold ids (`EventId`, `TraceId`, `LpmId`), never references. The
  `EventLog` is the only resolver.
- "Coverage" is `OccurrenceSet::by_event()`, a view, not a type.

## What is real and what is a placeholder

| Piece | Status |
|---|---|
| `core`: interning, log, `Lpm` over a rust4pm net, compiled firing rule, zip/XES IO | working |
| `align::MoveModel` for `Lpm` | working |
| `align::engine` | **baseline** best-first search; one optimal occurrence per start event, no model moves, token bound 2 |
| `discover::discover` driver, `Support`, `TraceSupport` | working |
| `discover::PlaceBased` | interface only, yields nothing |
| `fingerprint` | **placeholder** definition (before/inside/after activity counts, cosine) |

## Quick start

```rust
use lpmkit::core::{EventLog, LpmSet, SymbolTable};
use lpmkit::align::{align_all, MaxGap, UnitCost};
use lpmkit::fingerprint::fingerprint;

let mut symbols = SymbolTable::new();
let log = EventLog::from_xes_path("log.xes.gz", &mut symbols)?;
let lpms = LpmSet::from_zip_path("lpms.zip", &mut symbols)?;   // one *.pnml per LPM

let occ = align_all(&lpms, &log, &MaxGap { max: 2 }, &UnitCost);
let fps: Vec<_> = lpms.iter().map(|l| fingerprint(l, &occ, &log, 3)).collect();
```

Requires Rust 1.88+ (rust4pm's MSRV).
