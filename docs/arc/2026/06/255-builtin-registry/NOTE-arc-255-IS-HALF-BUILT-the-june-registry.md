> ⛔⛔ **STALE ON ITS HEADLINE NUMBERS — 2026-08-25.** This note's finding (the June registry is
> real; the arc is half-built) **stands and was correct**. Its *magnitudes* no longer are, and they
> are the lines a reader trusts fastest:
>
> | this note says | measured 2026-08-25 |
> |---|---|
> | "Registered production names today: **SIX**" | **189** `#[wat_intrinsic]` + 5 `#[wat_special_form]` |
> | "the carve reached **one home**" | `src/intrinsic/` is **8,443 lines**, ~20 homes |
>
> **STILL TRUE, and it is the arc's whole point:** the blanket-accept at `src/resolve/walk.rs`
> is LIVE (line **268** now, not 257), and its excuse is refuted — see the seam's THESIS section
> for the positive/negative control pair that proves the type checker does not do the leaf-level
> validation the comment delegates to it.
>
> ⚠ Validate any count here before quoting it. The one that catches people:
> `#[wat_intrinsic]` bare returns **27**; `#[wat_intrinsic` (the attribute takes an argument)
> returns **189**. A 7× error, and it is the same instrument-vs-question defect this arc keeps
> paying for.

# ⛔ NOTE — ARC 255 IS HALF-BUILT. The seam, the re-grounding note, and `255.1b-i`'s brief are all stale about it.

> Written the session AFTER the `2026-08-14` curare, at HEAD `a4c5bd1d`, before touching anything.
> Every fact below is one command run **this** session, cited.

## The finding, in one sentence

**A working intrinsic registry — with `metadata-of` answering live for registered intrinsics —
has been on disk since 2026-06-21, and yesterday's `255.1b-i` brief asks a rider to build its
type scaffold from scratch.**

## The proof it is LIVE, not a graveyard

Run this session, via the `wat` MCP against the built binary:

```
(:wat::runtime::metadata-of :wat::core::Bytes::to-hex)
⇒ #wat.core.Option/Some [{:name :wat.core.Bytes/to-hex
                          :arity 1
                          :kind        #wat.runtime.Kind/Intrinsic []
                          :defined-in  #wat.runtime.DefinedIn/Rust []
                          :layer       #wat.runtime.Layer/Substrate []
                          :purity      #wat.runtime.Purity/Pure []
                          :determinism #wat.runtime.Determinism/Deterministic []
                          :category    #wat.runtime.Category/Encoding []
                          :doc "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`. …"
                          :added "1.0.0"
                          :ret "the lowercase hex string, two chars per byte, no separators"}]
```

**That IS the LOCKED RECORD MODEL's Layer-1 baseline** (`name · arity · kind · pure ·
deterministic · defined_in · layer`), auto-derived at the registration site, answering through the
same verb a user form answers through — **minus `expand_time_legal`**.

## What is on disk (measured, with first-commit provenance)

| thing | where | landed |
|---|---|---|
| `crates/wat-doc/` — the doc leaf crate (`Purity`, `Determinism`, `DocExample`) | `crates/wat-doc/src/lib.rs` | `fea7ec15` 2026-06-21 (*"arc 255.1b-iv-a"*) |
| `crates/wat-macros/` — `#[wat_intrinsic]` / `#[wat_special_form]` proc-macros | `crates/wat-macros/` | June |
| `src/intrinsic/` — **1,374 lines**: the `inventory`-gathered registry | `mod.rs` 672 · `reflect.rs` 411 · `bytes.rs` 159 · `witness.rs` 84 · `special/` 48 | `e35badfa` 2026-06-22 |
| the baseline enums | `src/intrinsic/mod.rs:45–198` — `Kind` · `DefinedIn` · `Layer` · `RuntimeCategory` · `RuntimePurity` · `RuntimeDeterminism` · `Arity` | 2026-06-22 |
| **255.1b-iii — `metadata-of` answers for intrinsics** | `eval_metadata_of`'s intrinsic branch, `src/runtime.rs:12129`+ | **`7b99d123` 2026-06-21** |
| 255.1b-v — `show-source` + `render-doc` | `src/intrinsic/reflect.rs:160+` | June |
| 255.SF — special-form doc contract (`if`, `let`) | `src/intrinsic/special/` | `e35badfa` 2026-06-22 |
| eleven design/brief/note artifacts for iv-a/b1/b2/c, 1b-v, SF, SF-ii, doc contracts | this directory | 2026-06-22 → 06-27 |

**Registered production names today: SIX.** `:wat::core::Bytes::to-hex` · `Bytes::from-hex` ·
`render-doc` · `show-source` (intrinsics) + `:wat::core::if` · `:wat::core::let` (special forms).
Three more are test-only witnesses (`:wat::intrinsic::examples`, `variadic-args-measurement`,
`yields-witness`). Against ~332 + 141 + 678 sites of builtin knowledge, **the carve reached one home.**

## What is NOT built — the frontier, honestly

- **⛔ THE BLANKET-ACCEPT IS STILL LIVE.** `src/resolve/walk.rs:257` — `if is_reserved_prefix(head) { return true }`,
  comment intact (*"leaf-level validation is the type checker's concern"*). **255.1b-iv never landed.**
  The soundness hole this whole arc exists to annihilate is **open**.
- **The arc's own gates are DISARMED.** `eb680f3b` (2026-06-26) ignored 8 arc-255 tests to get the
  suite green, each reading *"unlock when we circle back to arc 255"*:
  `tests/wat_lang/probe_undefined_builtin_resolves.rs:17,:31` (**the 254.R gate**) ·
  `tests/reflection/probe_arc255_reflection_parity.rs:70,:82,:101,:107` ·
  `probe_arc255_ivc_metadata_plain_values.rs:68` · `probe_arc255_ivb2b_verify_examples.rs:33`.
  A ninth sits at `tests/types/probe_diag_typealias_leniency.rs:16` (*"arc 255 banked gate"*).
  **We have circled back.** Those nine ignores ARE the worklist, and they were written by a prior
  self as the unlock condition.
- **The LOCKED model's Layer-2/3 scaffold is genuinely absent** — grepped, zero hits:
  `Registration` · `MetaField` · `FnDef` · `DefDetail` · `NativeBuiltin` · `DefKind` · `ExpandTime`.
- **`FunctionBody::Native` is still never constructed** — and this one the prior note got RIGHT.
  `src/freeze.rs:749` states it explicitly and says it verified against every `Function { .. }`
  construction site. (I misread that comment as the opposite on first pass; corrected before it
  reached this note.)

## Two measured consequences the design should rule on

1. **There are now THREE purity enums.** `src/types.rs:235 Purity` · `crates/wat-doc Purity` ·
   `src/intrinsic::RuntimePurity` (which exists to *convert* wat-doc's, see the `match` at
   `runtime.rs`'s intrinsic branch). A brief that says "mint `Purity`" mints a **fourth**.
2. **`metadata-of` has TWO tables and two branches.** The intrinsic branch reads
   `crate::intrinsic::registry().lookup_entry(&name)`; the user branch reads `sym.binding_metadata`.
   The DESIGN's REFRAME (line 116) calls exactly this *"the opposite of seamless (two tables, two
   code paths)"* and rules **"The registry IS `sym`."** June built the other thing. That is not a
   verdict — the June path bought a real, working, proven reflection surface — but it is a
   **divergence from a LOCKED ruling that no artifact records**, and it decides what 1b-i even is.

## ★ WHY THIS WAS MISSED — the same failure, one layer up, by the document that names it

Yesterday's note ends with the rule *"Never grep for a name to test whether an arc is built. The
design may have killed that name. Grep for the thing it decided to build instead."*

Yesterday's re-grounding **was that grep.** It searched `src/` for the LOCKED model's names
(`Registration`, `MetaField`, `FnDef`, `DefDetail`), found nothing, and concluded 255.1a was the
only landed slice — **without ever reading the arc's own directory**, where `REALIZATIONS.md` (478
lines, R1) says in plain prose *"built and proven on `core::Bytes`, 255.1b-iii, commit `7b99d123`"*,
and where eleven June artifacts sit unopened.

Reading `DESIGN.md` in full was necessary and was done. **It was not sufficient**, because the arc's
built state does not live in its design — it lives in its REALIZATIONS, its commits, and its
disarmed gates.

**The rule that covers all three directions now:**

> Before briefing a stone, read the arc's **DESIGN** (what we meant), its **REALIZATIONS + commit
> log** (what we did), and the **disk** (what is there) — and treat any two of the three disagreeing
> as the finding, not as noise. A `git log --diff-filter=A` on the arc's own source directories is
> one command and would have caught this in ten seconds.

## What this note does NOT decide

It does not rule the direction. Two live readings, and they build different things:

- **(a) resume the June path** — carve the next homes with `#[wat_intrinsic]`, drive toward deleting
  the blanket-accept (1b-iv), un-ignore the nine gates as they go green. The scaffold question is
  moot; the frontier is the carve.
- **(b) land the LOCKED model's Layer-2/3 first** (`Registration`/`DefDetail`/`FnDef`/`MetaField`)
  and re-seat the June registry onto `sym`, per the *"registry IS `sym`"* ruling — then carve.

`255.1b-i`'s brief silently assumes (b) **and is written as if June never happened**, so it would
mint a fourth `Purity` and a second `Arity`. **It must not be struck as written.** Both readings
are the builder's call; the measurement is on the record so the call is made against the disk.
