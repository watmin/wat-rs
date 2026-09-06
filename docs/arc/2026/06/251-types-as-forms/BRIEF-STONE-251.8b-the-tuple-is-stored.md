# BRIEF — STONE 251.8b: the tuple is stored, not re-derived

Make `Identifier` hold `(namespace, name)` as fields instead of recomputing them from a flat string
on every read. **Behaviour must not change.** Read
`[[DESIGN-STONE-251.8b-the-tuple-is-stored]]` first — it carries the builder's model, the four
downstream guesses this unblocks, and the one signature that constrains the design.

## READ IN ORDER

1. **`crates/wat-reader/src/identifier.rs:85`** — the struct. `name: String`, `scopes:
   BTreeSet<ScopeId>`. **The site.**
2. **`:138-150` `namespace()`** — and its own doc: *"STONE 251.8a: the namespace is DERIVED from the
   spelling … 251.8b is where derived swaps for stored **behind this same signature**."* **That
   promise is row 3.**
3. **`:100` `Identifier::bare`** — the single construction chokepoint, debug-guarded. **The split
   happens here, once, and nowhere else.**
4. **`:132` `as_str`** — returns `&str`. **The design's one real constraint:** a borrow cannot point
   at a string assembled on demand.
5. **`:5-30`, the module doc on scopes** — Racket sets-of-scopes hygiene. **Orthogonal to the name.**

## SKETCH

```rust
pub struct Identifier {
    // the tuple, held:            wat.core/+  ->  ns "wat.core", name "+"
    //                             foo         ->  ns "$bound",   name "foo"
    ns:   String,
    name: String,
    flat: String,   // the spelling, so as_str()/leaf()/path() keep returning &str
    scopes: BTreeSet<ScopeId>,   // hygiene — NOT part of the tuple
}
// derived ONCE in `bare`. No accessor calls rfind. namespace() returns &self.ns.
```

## BLAST RADIUS

```
crates/wat-reader/src/identifier.rs    the representation + the accessor bodies
```

**Nothing outside the crate.** `name` is private, read 11 times, all inside its own accessors;
`pub name` is 0.

## STOP TRIGGERS

- **STOP-1 — BEHAVIOUR MUST NOT CHANGE.** Every accessor returns today's answer for today's inputs,
  including the answers that look wrong. This is a representation stone; it fixes nothing.
- **STOP-2 — `wat.core//` reads as `["wat.core/", ""]` today** and the builder's model says
  `[wat.core, /]`. **Do NOT fix that here.** It is a reader question, it is the next stone, and
  changing it makes row 9 unprovable. **Report what you observe.**
- **STOP-3 — no accessor may re-derive.** If a field is recomputed on read, the struct got bigger
  and nothing got better. Row 6.
- **STOP-4 — keep `as_str() -> &str` and every other `&str` signature.** If a stored tuple cannot,
  **STOP and report** — changing it ripples across 147 construction sites and is the builder's call.
- **STOP-5 — `scopes` is NOT part of the tuple.** Hygiene rides alongside. Folding it in makes a
  three-member name, which is the illegal shape.
- **STOP-6 — if any existing test goes red, STOP.** Capture the block verbatim; do not re-run.

## ⚠ TRAPS

- **This buys nothing visible today.** No bug closes, no output moves. Its entire value is that
  `#95`, the EDN codec, and killing keywords-as-heads all stop needing a guess afterwards. **Resist
  making it earn its keep by fixing something.**
- **Do not key anything on character case.** Builder's ruling. The casing heuristic in
  `ns_to_wat_path` is what this stone exists to make unnecessary — do not copy it.
- **147 sites construct through `bare`.** They are not edited; they are why deriving once is safe.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the accessor behaviour table, the `rfind` count, `-p wat-reader` — and report the
numbers.
