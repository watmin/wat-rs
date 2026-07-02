# Stone B — the payload: `#wat.core/Span` record + error faces → EDN + the `{:?}`-impostor wall

**Arc 296 closing strike, stone B.** Stone A (`09360465`) made "a wat type is EDN"
a structural fact from the foundation up (*FACTVM NON PACTVM*). Stone B spends
that: it makes the diagnostics real at *every face* a person or test looks
through, and on landing turns **R1 *NE SIBI OBSOLESCAT* → PROBATVM EST.**

## Why one campaign, not two stones

The error goldens embed spans AND assert the rust-debug `{:?}` impostor (59 test
files carry an error-struct-literal assert; ~340 touch the debug pattern).
Changing the span shape and converting `{:?}`→EDN each rewrite those same
goldens — **split in either order double-recaptures 59+ files.** So the
structural changes **bundle** and the golden recapture is **one fan-out, last.**
One sonnet owns the whole cascade (it causes the reds, it captures the greens).

## Pinned decisions

- **D1 — Span/Pos DERIVE `wat_edn::ToEdn`.** Not a hand impl. *FACTVM NON PACTVM* — a wat type is EDN by construction. (The stone-A hand impl in `span.rs` is deleted, replaced by `#[derive]`.)
- **D2 — `wat.core` namespace const → wat-edn.** Add `pub const CORE: &str = "wat.core";` to `crates/wat-edn/src/lib.rs` (the root, reachable by wat-reader AND wat). Span/Pos derive with `#[to_edn(namespace = wat_edn::CORE)]`. `wat/src/error_ns.rs::CORE` re-references `wat_edn::CORE` (`pub use wat_edn::CORE as CORE;` or `pub const CORE: &str = wat_edn::CORE;`) — **one source**, no drift (*FVNDAMENTVM NON MENTITVR*). The other error namespaces (CHECK/TYPE/…) stay in error_ns.
- **D3 — recapture is CAPTURE, DON'T GUESS.** Run the tests, replace each stale expected literal with the *actual emitted EDN* — never a hand-written guess. BUT the captured value must be **well-formed, sensible EDN**; if a "new" golden is malformed or nonsensical, that is a real bug — STOP, do not paper it over (*PROBATIO FLEXA MENTITVR* — a bent proof lies).

## The honest shapes (the target)

```clojure
;; a real range (wat chain — lexer/parser via with_end)
#wat.core/Span {:file "f.wat" :line 3 :col 8 :end #wat.core.Option/Some #wat.core/Pos {:line 3 :col 12}}
;; a point (rust chain — a Rust call-site, genuinely no end)
#wat.core/Span {:file "…/runtime.rs" :line 18900 :col 45 :end #wat.core.Option/None nil}
#wat.core/Pos  {:line 3 :col 12}
```

## The rooms (read in order)

**Phase 1 — structural (bundle; the cascade):**

1. `crates/wat-edn/src/lib.rs` — add `pub const CORE: &str = "wat.core";`.
2. `crates/wat-reader/src/span.rs` — the heart:
   - mint `pub struct Pos { pub line: i64, pub col: i64 }`, `#[derive(Clone, Debug, wat_edn::ToEdn)] #[to_edn(namespace = wat_edn::CORE)]`.
   - restructure `Span` → `{ file, line, col, end: Option<Pos> }`; `#[derive(Clone, Debug, wat_edn::ToEdn)] #[to_edn(namespace = wat_edn::CORE)]`.
   - `Span::new(file,line,col)` → `end: None` (kill the `end==start` sentinel); `with_end(...)` → `end: Some(Pos{..})`.
   - DELETE the hand `impl wat_edn::ToEdn for Span` (line ~148) — the derive replaces it. (Confirm the derive's `#[to_edn]` field handling emits `:end #wat.core.Option/… ` for the `Option<Pos>` — S1 proved this exact nesting.)
3. `wat/src/error_ns.rs` — `CORE` re-references `wat_edn::CORE` (single source).
4. The **7** `.end_line`/`.end_col` reads → `.end` (`Option<Pos>`); each was a range consumer (lexer/parser/diagnostics) — map to `.end.map(|p| p.line)` etc., or `.end` directly.
5. Retire the span serializers as the derive + `Span: ToEdn` subsume them: `splice_span` (14 callers), `span_to_map`/`span_to_edn` (panic_hook), `push_span_field`/`edn_span` (to_edn.rs). A derive-generated error with a `span` field now emits `:span (span.to_edn())` — the `push_span_field`/`is_span_type` special-casing in the derive DELETES (per FACTVM). **STOP if a caller needs behavior the derive can't give** (surface it).
6. Error faces → EDN: the 11 families (RuntimeError · CheckError · CheckErrors · TypeError · ParseError · MacroError · ConfigError · LoadError · ResolveError · StartupError · StdlibError) get **manual `Debug` + `Display`** that emit EDN via `to_wire_edn(self)` (replacing derived `Debug`). This is the `{:?}`-impostor wall: `{:?}` now emits EDN, so the rust-debug face is gone. Re-apply the RuntimeError manual Debug (reverted at baseline).

**Phase 2 — recapture (one fan-out, capture-don't-guess):**

7. `cargo test` → the goldens red (their expected strings are the old `{:?}` blobs / old span maps). For each: capture the **actual emitted EDN**, verify it is well-formed + sensible, replace the expected literal. Drive to green. ~59 files. STOP on any malformed capture (D3).

## Blast radius

wat-edn (const) · wat-reader (span.rs: Pos + Span + derives + new/with_end) · wat/error_ns.rs · the 7 `.end` reads · the ~25 serializer callers · the 11 error families' Debug/Display · ~59 golden files. The stone-A `impl ToEdn for Span` is DELETED (derive replaces).

## STOP triggers (rejection criteria)

- **STOP-1:** if Span deriving with `#[to_edn(namespace = wat_edn::CORE)]` does NOT emit `#wat.core/Span` (namespace path unresolved from wat-reader), STOP — report the exact error.
- **STOP-2:** if the `Option<Pos>` field does not emit `:end #wat.core.Option/Some #wat.core/Pos {…}` / `#wat.core.Option/None` (the honest shape), STOP — the derive's Option handling is the load-bearing assumption (S1 proved it; if it regressed, surface it).
- **STOP-3:** if retiring `splice_span` (or any serializer) requires behavior the derive can't produce (e.g. a caller that splices a span into a NON-derived value), STOP — surface the caller; do not hand-hack around it.
- **STOP-4 (D3):** on recapture, if a new golden value is malformed EDN or nonsensical (not merely different), STOP — that is a real regression, not a golden update. Never bend a probe to pass.

## Expectations (scorecard — fixed before the strike)

| # | what | command | expected |
|---|---|---|---|
| 1 | workspace builds | `cargo build` | Finished, 0 errors |
| 2 | Span emits the honest record | a probe/test on a with_end span | `#wat.core/Span {…:end #wat.core.Option/Some #wat.core/Pos {…}}` |
| 3 | a point-span is honest | a probe/test on `Span::new` | `:end #wat.core.Option/None` — NO `end==start` sentinel |
| 4 | errors are EDN at every face | `format!("{:?}", err)` and `format!("{}", err)` on a sample error | both structured EDN, no rust-debug blob |
| 5 | full suite green | `cargo test` | 0 failed (save the 7 pre-existing `wat_dispatch` flakes) |
| 6 | serializers retired | `grep -rn "splice_span\|push_span_field\|span_to_map\|span_to_edn\|edn_span" src/ crates/` | empty (defs + callers gone) |
| 7 | one source for wat.core | `grep -rn '"wat.core"' src/ crates/` | only the wat-edn const literal; error_ns re-references it |
| 8 | goldens captured, not guessed | spot-read 5 recaptured goldens | well-formed EDN matching actual emission |

**Runtime prediction:** 60–120 min (the recapture dominates). **Trap-doors:** a `.end_line` read that assumed a real range (now `None` for point-spans — handle the `Option`); a serializer caller that isn't derive-reachable (STOP-3); a golden whose `{:?}` had non-error content the recapture must preserve; the `Display` faces (some families may have meaningful human Display worth keeping alongside the EDN Debug — confirm the DESIGN's "Display→EDN" doesn't erase a wanted human message; if a family's Display is load-bearing-human, surface it).

## On landing

R1 *NE SIBI OBSOLESCAT* → **PROBATVM EST** — the error is EDN at every boundary a person or a test looks through, and spans are typed `#wat.core/Span` records with honest `Option` ends. `push_span_field` self-deleted. The arc's prophecy fulfilled.

## Reference

- S1 (`struct_derive_emits_namespaced_tagged_record_with_optional_nested`) — the proven derive-on-struct-with-Option<record> shape to mirror.
- Stone A (`09360465`) — the derive reaches wat-reader; the pattern the recapture builds on.
- *VERA FACIES VERA VOX* (interstitial) — the `{:?}`-impostor named; this stone kills it.
