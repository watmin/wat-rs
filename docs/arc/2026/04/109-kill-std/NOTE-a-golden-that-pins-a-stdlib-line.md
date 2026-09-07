# NOTE — a golden that pins a stdlib `.wat` SOURCE LINE breaks on unrelated edits

**Found 2026-09-07**, RELAND 5 of the match-arm strike. Kin of
`NOTE-a-golden-that-pins-a-rust-line-number.md` — the same defect in wat clothing.

## What happened

The arm migration rewrote clauses in `wat/service.wat` and moved its lines by 29.
Four `peers_bijection` goldens went red. Everything else was byte-identical:
same `ProgramBodyEvalFailed`, same `MalformedTemplate`, same `:reason` text,
same `:col`. Only the stdlib span moved:

```
golden   :file "wat/service.wat" :line 864 / 881   :end :line 871 / 889
actual   :file "wat/service.wat" :line 893 / 910   :end :line 900 / 918
```

Recaptured with `UPDATE_EDN=1`. The diff is those four `:line` integers and
nothing else. Fixture-file spans (`tests/services/probe_arc278_peers_bijection_case*.wat`
`:line` 44 / 70) did not move.

## Why it is worth a NOTE

**A golden that pins a stdlib source line measures `wat/service.wat`'s shape, not
the program's behaviour.** Any edit to the stdlib — including a semantically
inert delimiter flip — invalidates the pin. The next stdlib edit will break
these four again.

The substrate already normalises *Rust* spans for this class:
`normalize_rust_source_span_lines` in `src/lib.rs` zeros `:line` on a
`#wat.core/Span` whose `:file` ends in `.rs`. It does **not** touch a span
whose `:file` is `wat/service.wat`. That is why these four fired and a
`src/*.rs` pin in the same fixture would not have.

## Standing rule (do not re-propose the retracted half)

The sibling NOTE retracted "compare `:file`, drop `:line`". Arc 296 ruled
**KEEP PINNING THE SPAN** — a pin discriminates the emitter; dropping it
makes a different call site raising the same error kind silently green.
That ruling is about `src/*.rs` `rust_caller_span!()` coordinates. A stdlib
`.wat` span is the same kind of pin: it names *which form in service.wat
emitted this error*. Recapture it. Keep pinning it.

What this NOTE adds is only the wat-side observation, and the recapture
check: confirm the new line is the same form in the same file, and that
only its position moved. Do not silently recapture. Do not extend
`normalize_rust_source_span_lines` to `wat/*.wat` as a side effect of
fixing a red — that is a later ruling, and it would be the retracted
option wearing stdlib clothes.

## The four

| golden | `:line` | `:end :line` |
|---|---|---|
| `probe_arc278_peers_bijection__case1_old_spelling_missing_ephemeral_is_rejected.edn` | 864 → 893 | 871 → 900 |
| `probe_arc278_peers_bijection__case4_form_spelling_missing_ephemeral_is_rejected.edn` | 864 → 893 | 871 → 900 |
| `probe_arc278_peers_bijection__case2_old_spelling_undeclared_peer_is_rejected.edn` | 881 → 910 | 889 → 918 |
| `probe_arc278_peers_bijection__case5_form_spelling_undeclared_peer_names_the_surface.edn` | 881 → 910 | 889 → 918 |
