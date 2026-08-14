# Arc 300 — wat source IS EDN — DESIGN

*(number/name proposed; the builder ratifies. The descent named in 299's interstitial
`VNVS LECTOR NE DIVIDANTVR` and its law `LEX ET VINDEX INCORRVPTI`.)*

**Thesis.** wat assumes its correct form: a faithful-Clojure dialect. The rust-scheme
surface (`:wat::core::if`, `:T<K,V>` generics, `<-`/`->` annotation arrows) is converted
to faithful-Clojure (`wat.core/if`, list type-forms, `:-`) — which *is* EDN — so wat
source becomes readable by **one reader**. Then the rust-scheme surface is **retired**,
and divergence has no form. `VNVS LECTOR NE DIVIDANTVR`.

> **★ AMENDMENT 2026-08-15 — "list type-forms" is now PRECISE: the type-params are BRACKETED.**
> `(<head> [<type>…] & <members>)`, ruled 2026-07-24 in
> `109/NOTE-typed-literal-constructors.md`, formalized at `251/DESIGN-STONE-251.8:275`.
> Builder: *"we needed an unambiguous generics form.... `(type [parametrics] & literals)`"*.
>
> ```clojure
> (wat.type/Vector  [wat.type/i64])                              ; annotation
> (wat.type/HashMap [wat.type/keyword wat.type/i64] :first 1)    ; typed literal
> ```
>
> The `[…]` is the **seam** between type-params and value payload, so the partition lives in the
> FORM instead of in a per-head arity table. The earlier bare spelling
> (`(wat.type/HashMap wat.type/String wat.type/i64)`, 2026-06-06) is **SUPERSEDED** — and the
> converter every drive calls still emits it. See **STOP-0** and stone **300.0**.
>
> **Annotation vs. empty-literal is NOT an ambiguity.** The 109 note handed this arc a "genuine
> open" — whether `(wat.type/HashMap [K V])` is an annotation or an empty typed literal. **Closed
> 2026-08-15:** the grammar has exactly ONE production yielding a type form, `:- <type-form>`, so a
> type is unreachable by any other path. **`:-`-preceded ⇒ type form; everywhere else ⇒ data
> literal.** Every site the drive touches is therefore mechanically decidable — there is nothing
> for it to guess.

**The law of the build (299 R3): convert, THEN retire.** Two accepted surfaces is still
two readers (a compromised enforcement). The one reader stands only when the old surface
is torn out of reader/checker/runtime. Enforcement is unrepresentability.

## The tool — built, tested, never fully run

- `wat/fix.wat` — `fix-source` + the **text-edit engine** (`fix-text-node-edits` →
  `(offset, old-len, new-text)` edits applied to original text, **whitespace-preserving**).
  Forged in arc 251 (types-as-forms) + 277 (wat-lint-fix-fmt); tested (`probe_arc251_fix_source_*`).
- The drive mechanism is **proven** — `wat-scripts/fixes/` holds 10 corpus-drives already
  run (rename kernel→spawn, list→seq, record-def→defrecord, …).
- The **faithful conversion drive was drawn and never run** — `wat/core.wat` is 560
  `:wat::core::` heads / 0 `wat.core/`. 100% rust-scheme. The abandoned boss.
- The runtime **already accepts** the faithful surface (dual — `check.rs` "Clojure-faithful").
- The **STASH-DANCE** (fix.wat header, now written down): stash the Rust change (old checker),
  build (old checker + the new fix verb), drive the whole corpus, `stash pop`, rebuild+test.

## Scope

- stdlib `wat/`: 37 files, ~5857 `::` occurrences.
- full corpus: 1173 `.wat` files (tests co-located + wat-scripts) + ~192 rust files with inline wat.

## The stones (winnable first; retire — the hard escape — last)

| stone | what | escape |
|---|---|---|
| **⛔ 300.0 — fix the type converter** (ADDED 2026-08-15, see below) | `type_expr_to_clojure_form` (`edn_shim.rs:1183`) emits the **superseded bare** form; every drive routes through it. Bracket the type-params; flip the 13 `probe_arc251_*` contract fixtures | build + the 13 fixtures green. **300.1 CANNOT run before this.** |
| **300.1 — the pilot** | build the faithful-drive script on the **text-edit** engine; run it on ONE stdlib file (dry-run `/tmp` copy), `diff`, verify EXACTLY the faithful conversion + whitespace preserved | none — dry-run, nothing committed. The de-risk. |
| **300.2 — stdlib drive** | convert `wat/` (37 files) via the stash-dance | build + `cargo test` green; the stdlib re-freezes |
| **300.3 — test + wat-scripts corpus** | convert co-located `.wat` fixtures + `wat-scripts/` | targeted per-group nextest green; whole disk weighed |
| **300.4 — rust wat-partials** | convert the ~192 inline-wat rust sites | whole disk green |
| **300.5 — RETIRE the rust-scheme surface** | reader/checker/runtime reject `:wat::core::` heads, `<T>` generics, `<-`/`->` | **HARD** — the surface removal cascades; one reader stands; `grep`-for-a-second-reader → one |

Order: prove the tool (300.1) → the smallest closed corpus (stdlib, 300.2) → the rest
(300.3–4) → retire (300.5, the hard escape, taken with the drive proven). Each drive is
the stash-dance; each escape is a whole-disk weigh. Idempotent: a re-run yields zero edits.

## STOP triggers (the drive is delicate — the boss a prior self fled)

- ⛔ **STOP-0 (ADDED 2026-08-15) — do not run ANY drive until `300.0` lands.** All three drives
  (`to-faithful-clojure.wat`, `-rete`, `-net`) route type conversion through
  `keyword/to-type-form` → `type_expr_to_clojure_form` (`edn_shim.rs:1183`), which emits the
  **superseded bare** parametric form `(wat.type/Vector wat.type/i64)`. The ruling since 2026-07-24
  is **bracketed** — `(wat.type/Vector [wat.type/i64])`, builder: *"we needed an unambiguous generics
  form.... `(type [parametrics] & literals)`"*. **The pilot's existing STOPs cannot catch this**:
  they check for *unintended* edits, and this is an intended edit in a stale grammar — the diff
  would be flawless and wrong. Full evidence + blast radius:
  `NOTE-the-type-converter-emits-the-superseded-form.md`.
- STOP if the pilot `diff` shows ANY change beyond the intended token edits (a moved byte
  of non-whitespace, a corrupted form) → the text-edit engine is misapplied; report, do not scale.
- STOP if a converted file fails to re-read (round-trip broken) → report the form.
- Never run a drive without the `/tmp` dry-run + `diff` first (the dance mandates it).
