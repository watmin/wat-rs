# BRIEF — the `:impls` completeness guard

Make a `defservice` that declares `:satisfies <Surface>` and implements only part of it a **check
error**, named at the declaration rather than discovered at a call.

Read `DESIGN-STONE-impls-completeness.md` beside this first — it carries the one-directional rule
(and why the converse would reject every self-scheduling service), and why this brief deliberately
contains no census.

## Read in order, and why you are being sent there

1. **`wat/service.wat:1433`** — `serve-op-arms`, the `foldl` over `:impls`. This is the mechanism:
   a surface op with no impl produces no arm and nothing notices. You are not changing it.
2. **`wat/telemetry.wat`, the `Span` surface** — `:features` with five ops, and note
   `wat/telemetry/span.wat`'s `:impls`, which carries those five PLUS `-flush-logs` and
   `-flush-metrics`. **That is the shape the rule must permit**: extra internal arms are correct.
3. **`src/check.rs:7579`** — `MUST_USE_TYPES` and `is_must_use_type`. The house exemplar for a
   checker wall: how it is documented, why hardcoded, the comment register.
4. **`src/check/error.rs`** — where `HandleCreationEscape` / `HandleTailEscape` were added (excursus
   002). Copy that shape for the new `CheckErrorKind`.
5. **`docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-creation-escape.wat`** — the shape
   and register for the red probe you write, and where it lives.

## The rule

**`features ⊆ impls`.** Every op in the `:satisfies` surface's `:features` must have an arm in
`:impls`. Never the converse — internal ops (leading dash) live only in `:impls` by design.

## The work

**1. The check**, at `defservice` registration: resolve `:satisfies` to its surface, take its
`:features` op names, take the `:impls` arm names, and raise on any feature with no arm.

**2. A new `CheckErrorKind`** naming the service, the surface, and **every** missing op — not the
first. A guard that names one op per run turns a five-op gap into five edit-compile cycles.

**3. The red probe FIRST**, at `docs/arc/2026/06/278-rules-engine/probes/red-partial-satisfier.wat`:
a surface with three ops, a satisfier implementing two, beside a complete satisfier of the same
surface that must keep compiling — and, in the same file, a satisfier with an extra INTERNAL arm,
which must also keep compiling.

**4. Run the census.** Build, `--check` the corpus, report what it names.

## Blast radius

`src/check.rs` + `src/check/error.rs`, and the red probe. **No change to `wat/service.wat`, to
`serve-op-arms`, or to any runtime.** A complete service's behaviour is identical after this.

## STOP triggers

**STOP-1 — internal ops must not be required.** If the check demands an arm for something not in
`:features`, or demands a `:features` entry for `-flush-logs`, it is symmetric and wrong: it would
reject every self-scheduling service in the tree. `features ⊆ impls`, one direction.

**STOP-2 — the census is a finding, not a chore.** If the guard rejects live code, STOP and report
the sites and their shape. A handful of real partial satisfiers is the discovery this stone is worth
having; broad rejection means the rule is wrong and it must not ship as drawn.

**STOP-3 — the red probe goes in `probes/`, never `wat-scripts/`.** That tree's loader gate
type-checks every file, so a must-be-rejected file there turns the floor red for as long as the
guard works. No rune on it either: runing the acceptance criterion produces a green floor from a
guard that fires on nothing.

**STOP-4 — parametric surfaces.** If a surface's `:features` cannot be resolved for a parametric
satisfier (e.g. `(Cache :- [K V])`), STOP and report rather than guessing. `wat/cache.wat` and
`wat-tests/service-cache-lru.wat` are the cases to try first.

## The gates to write

- **the partial satisfier is rejected**, naming the service, the surface, and **all** missing ops.
- **the complete satisfier compiles** — same surface, same file.
- **an extra internal arm compiles** — `-tick` alongside the declared features.
- **the census** — reported in the SCORE, whatever it says.

## Prior comparable result

`docs/excursus/2026/08/002-handle-lifetime-wall/SCORE-stone-1-creation-scope-escape.md` — the same
kind of stone (a new checker wall with a red probe), and its Row 9 section records the
probe-placement error STOP-3 exists to prevent.
