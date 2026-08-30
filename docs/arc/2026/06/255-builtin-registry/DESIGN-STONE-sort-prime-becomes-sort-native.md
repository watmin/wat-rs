# DESIGN — STONE `sort'` → `sort$native` (option C: the rename, not the homing)

> **Builder, 2026-08-30:** *"we have been moving to using `name$native` instead of `name'` to
> denotate a native impl… `$oracle` is for wat defined."* → *"can we impose the sort fn is pure,
> deterministic, total?"* → measured refutation → ***"do C now… and draw A as the next stone"**.*

## Why

`:wat::core::sort'` is the last verb wearing `'` as a **native-impl marker**
(`NOTE-the-prime-suffix-does-three-jobs-and-native-replaces-one.md`). The established convention is
`$native`, already applied to the five `:wat::rete::` firing verbs and stated in the retired-name
lint's own header: *"public names are native, the wat reference is `$oracle`."* `sort'` is the
straggler.

It also carries **five earned `rune:lint(retired-name)` exemptions** — an exemption is only earned
while the name is one a user cannot type. Rename it and the lint stops seeing it at all, so all
five retire. One of those runes went missing when its arm relocated and took the floor red on
2026-08-30 (`[[feedback_a_co_located_rune_is_attached_to_a_line]]`); a name needing no exemption
cannot lose one.

## What ships

```
:wat::core::sort'  ->  :wat::core::sort$native
```

1. **`wat/core.wat`** — 4 call sites (`:1522 :1530 :1537 :1546`) + 2 prose lines (`:1513-1514`).
   ⛔ Via the **wat-fix codemod** (R21), never a hand-edit.
   ★ **`wat/core.wat` is the ONLY caller in the corpus** — measured; every other site in `wat/`,
   `tests/` and `wat-scripts/` calls the public `sort` / `sort-by` defclauses.
2. **`wat-scripts/scratch-pad/255-probe-can-a-user-make-sort-effectful.wat`** — 2 call sites, same
   codemod run (the `every_wat_scripts_file_loads` gate parses it, so it must move with the name).
3. **Five Rust sites**, each losing its now-unearned rune:
   `src/collection/transform.rs:282` (`const OP`) · `src/runtime.rs:6023` (dispatch arm key) ·
   `src/check.rs:20272` (TypeScheme key) · `src/macros/eval.rs:505` (expand-time list) ·
   `src/rete/purity.rs:2046` (`KNOWN_UNREVIEWED`).
4. **One `RETIREMENT_TABLE` row** (`src/remedy/retirement.rs`).

## THE ONE CONTRACT DECISION — pinned

**It is a RENAME, not an alias.** `:wat::core::sort'` stops dispatching entirely and becomes a
`RETIREMENT_TABLE` hit — a check-time error naming `:wat::core::sort$native` as the remedy.
**No dual-spelling arm.** Precedent: arc 255 Stone C did exactly this for `:wat::core::i64::*`
(`src/intrinsic/i64.rs`'s header records it). The rete family's `"…$native" | "…"` two-pattern arm
is NOT the precedent here — that arm exists because the public FQDN is also a first-class wat `Fn`,
which `sort'` is not.

⚠ The public surface **does not move**: `sort` and `sort-by` are wat `defclause`s in `core.wat` and
are untouched. Nothing a normal user types changes.

## What stays, deliberately

- **The hand-registered `TypeScheme` in `check.rs` stays** (renamed only). Measured: homing does not
  retire a TypeScheme — homed `length`/`range` still carry one while `nth`/`reverse` do not. Typing
  and registration are independent axes here.
- **The `KNOWN_UNREVIEWED` row stays** (renamed only). That ratchet fires on *homing*, not renaming.

## Out of scope = REJECTED (not deferred)

- **Homing it into the registry** — that is **STONE A**, drawn as this stone's sibling
  (`DESIGN-STONE-A-the-classifier-cannot-follow-a-captured-fn.md`). Not "later": the next stone.
- **Imposing pure ∧ det ∧ total on the comparator** — ruled and then REFUTED by measurement this
  session (`wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat`):
  the classifier default-denies `sort-by`'s free-variable comparator, so the gate would break every
  `sort-by` caller. Belongs to STONE A, which removes the blindness first.
- **The other two prime jobs** (`readln'`; the positional-ctor family) — builder: *"we'll deal with
  the rest of primes later."* Affirmatively cut from this stone.

## The four questions

- **Obvious? YES** — one verb, one spelling, matching a convention already applied five times.
- **Simple? YES** — a rename plus a retirement row. No new mechanism, no behaviour change.
- **Honest? YES** — the old spelling errors with a remedy instead of silently vanishing, and five
  exemptions stop being claimed for a name nobody can type.
- **Good UX? YES** — the public `sort`/`sort-by` surface is untouched; the only caller is the
  substrate's own `core.wat`.

## Acceptance

| what | command | expected |
|---|---|---|
| no primed sort survives in code | `grep -rn "sort'" src/ wat/ wat-scripts/ --include=*.rs --include=*.wat` | only `wat-scripts/fixes/reclaim-service-fixture-names.wat:19` (historical prose in a recorded migration) |
| ⚠ ~~the five runes are gone~~ | ~~`grep … \| grep sort`~~ | ⛔ **THIS BAR WAS WRONG AND COULD NEVER PASS** — the `RETIREMENT_TABLE` row must hold `":wat::core::sort'"` verbatim, so it is itself a lint hit needing a rune. Corrected bar: **no rune survives at a site whose name no longer contains a prime** — five retired, one born, and the one born is structural. See `NOTE-a-retirement-row-must-hold-the-shape-its-own-lint-hunts.md`. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]` |
| old spelling teaches | a `.wat` calling `(:wat::core::sort' …)` under `--check` | check-time error naming `:wat::core::sort$native` |
| ⚠ public surface intact | ~~`wat tests/resolve/probe_arc251_ordering_surface.wat`~~ — that fixture has NO `:user::main` (it is consumed by a Rust test), so the command errors for a reason unrelated to this stone. Corrected: a standalone program exercising `sort/1`, `sort/2` and `sort-by` | `[1 2 3]` · `[3 2 1]` · `[3 2 1]` — **verified** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
