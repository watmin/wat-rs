# Arc 245 — wat-corpus warding (STUB)

**Status:** ⏸ **STUBBED — BLOCKED, enabled by arc 237's closure.** Not yet designed; this stub banks the intent. Opens when **237.9 (INSCRIPTION) ships** and the numeric+equality stdlib surface is stable. Stubbed 2026-06-03 from a dialogue at the close of Stone 237.8b.

## Why this arc — the asymmetry it kills

`src/` has **warded homes** carrying vigilatum stamps (failure-classes annihilated, L1+L2=0, proof inscribed in the code). The `wat/` stdlib — **the surface users actually call** — is **untrusted-by-default**. That is the *src-warded / wat-untrusted asymmetry*, and arc 244 established the doctrine: **asymmetries must clear a very high bar; an unwarded member of a warded family is a defect, not a quirk** (`feedback_asymmetries_meet_high_bar`). The stdlib deserves the bar more than almost anything — it is the foundation every program composes on. This arc raises the whole wat corpus to a defined bar and inscribes the proof.

## Why BLOCKED until 237 closes

1. **Do not ward a file you are about to rewrite.** 237.8c (the equality grid) re-churns `wat/core.wat` — it adds the `=`/`not=` defclauses alongside the `+`/`-`/`*`/`/` ones 237.8b just shipped; 237.8d/237.9 continue settling the numeric/dispatch surface. A vigilatum stamp on a churning file is a lie ("warded as of X" while X changes next stone). `core.wat` is the most important file and the worst one to ward mid-237.
2. **Winding discipline.** Arc 237 is the active context (8b shipped; 8c → 8d → 9 remain, close + within reach). Opening this now is the arc-jump `no-regression-until-arc-done` forbids. Finish the chain; start this fresh on a stable stdlib.

## Scope (crawled 2026-06-03)

**61 files, ~11,425 LOC.**
- `wat/` stdlib — **26 files**: `core.wat`, `edn.wat`, `list.wat`, `Record.wat`, `runtime.wat`, `stream.wat`, `test.wat`, `holon.wat` + `holon/*.wat` (12), `kernel/*.wat` + `kernel/services/*.wat` (7).
- `wat-tests/` — **35 files**.

## The opening design stone — the instrument question

The vigilia 8-spell + the ward spells were built for **Rust homes**; several are Rust-specific (clippy, borrow-shape, the `src/<noun>/` lift). wat is a different language (typed Lisp), and `wat/` files are **stdlib source, not "homes"** in the lift sense. So *"bar-raised for a wat file"* must be **defined before any cast**:

- **Which grimoire spells transfer?** Likely yes: **intueri** (does the code speak / naming), **conferre** (doc-vs-territory fidelity — do the wat comments match the wat behavior), **struere** (structural soundness), **consonare** (voice). Likely no / needs adaptation: clippy-based and Rust-borrow-specific checks.
- **Is the instrument an intueri-named adapted procedure** — a *"wat-ward"* — or a curated subset of the existing wards cast on wat source? (Name it via a real intueri cast when the arc opens.)
- **Same bar for `wat-tests/` as `wat/` stdlib?** The stdlib is production (highest bar). Tests should be correct + honest, but the warding instrument and bar may differ.
- **Selective vs blanket?** Warded homes grew by **selective** lift-and-ward (`feedback_selective_lift_and_ward`). For the **stdlib**, "all of it at the bar" is defensible (the foundation should be uniformly trustworthy). For **tests**, selective may be the honest call. Decide in the design stone.

## Provisional slicing (to be confirmed in design)

1. **245.0** — design: name the instrument (intueri), settle which spells transfer + the per-file bar, confirm scope/slicing.
2. Stdlib core files (`core.wat`, `list.wat`, `Record.wat`, `runtime.wat`, `stream.wat`, `edn.wat`).
3. Stdlib `holon/*` family.
4. Stdlib `kernel/*` family.
5. `wat-tests/` (per the bar decided in 245.0).
6. INSCRIPTION + the wat-corpus-warding doctrine.

## Enabled-by

**Arc 237 closure (237.9 INSCRIPTION).** When 237 closes, 237.9 should flag arc 245 as unblocked, and this stub graduates to a full DESIGN. Until then: do not open; do not partial-ward `wat/core.wat`.
