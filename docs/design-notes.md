# Design notes

Decisions taken so far and considerations deferred to later. Each deferred item
records what was decided for now, why, and what would make us revisit it.

## LPM identity: name, no id (for now)

**Decision.** An `Lpm` is identified by its `name`, the file stem of the PNML it
was read from. There is no numeric id. Names must be unique within one archive;
the reader rejects duplicates (e.g. `a/x.pnml` and `b/x.pnml`).

**Why.** It is not yet clear whether models will travel on their own or always
inside a set, which decides where an id should live.

**Revisit when** models are referenced from many other values (occurrences,
scores, results). Then consider:

- A dense `LpmId(u32)` that indexes into a `Vec`: cheap to copy, compare, hash
  and store, unlike a `String`.
- Keep the name as a field and build a `HashMap<String, LpmId>` while reading,
  for lookup by name and for joining external data.
- Where the id lives:
  - *Outside the `Lpm`*, as its position in an `LpmSet` that hands out
    `(LpmId, &Lpm)`. No redundancy, but needs the set at hand.
  - *Inside the `Lpm`*. Convenient when a model travels alone, but goes stale
    when sets are filtered, sorted or merged.
- Id order: archive order depends on the tool that built the zip; sorting by
  name is stable across tools but puts `lpm_10` before `lpm_2` unless a natural
  sort is used.
- Whether names should be the full path inside the archive instead of the file
  stem, to allow models with the same file name in different folders.

## Additional model information from a CSV (deferred)

**Decision.** A model archive may contain a CSV with extra information about the
models. The reader currently **ignores** it, since nothing uses it yet.

**Why deferred.** It is unclear which columns the CSV will have and how the
values will be used, though comparing models by them is likely.

**Assumed format** (when picked up): header row; one column names the model
(matching the PNML file stem); other columns are values for that model.

**Options for storing it:**

| Option | Shape | For | Against |
|---|---|---|---|
| A. Text map per model | `info: BTreeMap<String, String>` on `Lpm` | Simplest; any columns | Callers parse values; column typos show up only at runtime |
| B. Typed values per model | rust4pm `Attributes` / a small `Value` enum | Numbers come back as numbers; matches XES attributes | Type guessing (`"1"`: number or text?) |
| C. User-defined struct via serde | `Lpm<I>` with `info: I: Deserialize` | Columns checked by the compiler | Type parameter spreads everywhere; adds `serde`; one struct per CSV layout |
| D. Separate table keyed by model | reader returns models plus an `InfoTable` (columns indexed by id) | `Lpm` stays minimal; later computed values (support, fingerprints) fit the same table | One more type to pass around |
| E. Raw | return the CSV text unparsed | No decisions now | Defers the problem |

Leaning: **D** (needs an id, see above) or **A** as the simplest start. If models
are compared by these values, typed columns (B, C, or D with typed getters) are
worth it.

**Validation to decide on:** more than one CSV in an archive; a row naming a
model that is not in the archive; two rows for the same model; models with no
row; how empty cells are treated.

**Open questions:** Where does the CSV come from, and are its columns fixed?
Will code rank or filter models by these values, or only display them?

## Model representation: Petri net only (for now)

**Decision.** An `Lpm` holds a `PetriNet` (`net`) and nothing else. Models in
other formats would be converted to a Petri net when they are read.

**Why.** Petri nets are the common target that the other formats translate
into, so every algorithm only has to handle one kind of model. No other input
format exists yet, and `Lpm`s are only created in `core::io`, so changing the
representation later is a contained change.

**Revisit when** the first non-PNML input arrives, when discovery starts
building candidates in another form (e.g. process trees), or when episodes or
sequences are added (see below).

**What rust4pm offers** (checked against the local checkout, `process_mining` 0.6):

- `ProcessTree` exists and converts to a Petri net (`ProcessTree::to_petri_net`,
  `From<ProcessTree> for PetriNet`), but there is no file import for it (no PTML).
- BPMN is not supported: a parser and a translation would have to be written.

**Options:**

| Option | Shape | For | Against |
|---|---|---|---|
| 1. Petri net only (current) | `net: PetriNet`; convert on read | One semantics for all algorithms; trees convert with one rust4pm call | Original structure (tree operators, gateways) is lost, e.g. for display |
| 2. Enum of formats | `model: Model { PetriNet(..), ProcessTree(..), .. }`, `#[non_exhaustive]` | Keeps the original; algorithms match or convert | Every algorithm needs a `match`; closed set |
| 3. Trait for what algorithms need | `Lpm<M: LpmModel>` exposing labels and firing behaviour | Open to new formats; fits "traits are design-space axes" | Type parameter spreads; mixed sets need `Box<dyn ..>` |
| 4. Original plus compiled form | keep the source model (option 2) plus a Petri net or compiled firing rule | Display from the original, algorithms from one form | Two representations to keep in sync |

Leaning: stay with 1 now; option 4 is the likely end state.

**Points specific to LPMs:**

- LPM discovery in the original work (Tax et al.) grows candidates as process
  trees (sequence, choice, parallel, loop). If discovery works that way, the
  tree is the natural working form and the Petri net is derived for alignment.
- Tree-to-net translation introduces silent (τ) transitions; alignment must
  then support moves on silent transitions.
- Not all BPMN translates cleanly (e.g. OR-joins); such models would have to be
  rejected or approximated.

## Beyond LPMs: episodes and sequences (future scope)

**Consideration.** The library may later cover other local behavioural patterns
besides LPMs:

- **Sequences** (sequential patterns): an ordered list of activities, usually
  matched with gaps allowed between consecutive items.
- **Episodes** (frequent episodes): a partial order over activities, ranging
  from serial (a total order) through parallel (no order) to general partial
  orders, usually matched within a time or event window.

**What this affects:**

- **Representation.** Both can be expressed as Petri nets (a serial episode or a
  sequence is a chain; a parallel episode is an AND-split/join; a general
  episode has one place per ordering constraint), so option 1 above would still
  work. But they have much simpler direct forms (`Vec<Symbol>`, a DAG over
  labels) that allow much faster matching than general net alignment. That
  argues for option 2 or 4, where each kind keeps its own form.
- **Matching semantics differ, not just the shape.** Episodes are typically
  counted per window or by minimal occurrences; sequences by containment with
  gap limits; LPMs by alignment-based occurrences. Gap and window constraints
  should then be shared parameters, not LPM-specific (the earlier layout already
  had a `GapConstraint` axis for alignment).
- **Naming.** If episodes and sequences share the same pipeline (read, match,
  measure, fingerprint), `Lpm` becomes one kind of a more general concept
  (e.g. `Pattern`), and the crate name and description would need to follow.
- **Comparing across kinds.** Fingerprints and quality measures would ideally be
  defined on occurrences, not on the model, so they work for every kind.
