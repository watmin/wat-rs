# AMEND — STONE 255.51: STOP-1 ruled; finish the stone

**Builder, 2026-09-26: F1.** `wat-scripts/fmt/fixtures/generic-fn.wat`'s lambda becomes a real generic `fn`, written
in the **target spelling** (the builder's own form; the file may mix spellings, and the checker accepts this one):

```clojure
(wat.core/fn :- [T]
  [acc :- wat.core/i64
   x   :- T]
  :- wat.type/i64
  acc)
```

Replace the inner `(:wat::core::fn :- [:wat::core::i64] …)` form with exactly that (keep the enclosing `defn`, the
`foldl`, `0` and `xs`). Measured by the orchestrator: this shape passes `wat --check` today (rc 0). The return type
is `wat.type/i64` because the return position does not yet resolve `wat.core/i64` as a type; that is a known
clojurification item, not this stone's. Do not change it here.

Then finish the stone: run the release floor (`scripts/floor.sh`), clippy
(`cargo clippy --release --all-targets -- -D warnings`), census (`scripts/replay/census.sh --diff`), and delta
(`scripts/replay/delta.sh`). Append the results to the SCORE. Commit the fixture and the SCORE on `main`
(`git add -- <paths>`). **Do not push.** Then `pulsare_yield kind=scored`.

STOP (unchanged doctrine): any other red, do not re-run; quote the block verbatim; report.
