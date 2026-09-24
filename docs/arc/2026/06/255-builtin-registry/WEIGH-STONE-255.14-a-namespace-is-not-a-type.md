# WEIGH — STONE 255.14: PART 1 ACCEPTED, PART 2 STOP ACCEPTED. ⭐ **UnresolvedReference 40 → 0.**

Weighed against `9ec9423fc` by independent re-run. Tree clean.
Converted floor **107 → 78**. Trajectory **3 747 → 416 → 283 → 161 → 112 → 107 → 78**.

## ⭐ PART 1 — THE RULING, IMPLEMENTED

The seven spawn builder declarations now use the namespace join (`wat/spawn.wat:124-168`, verified).
Recorded codemod `wat-scripts/fixes/spawn-builder-namespace-join.wat` — whole-token renames,
idempotent, replay fixture + oracle. ⭐ **Three negative controls byte-identical on the dry run**,
including `wat/cache.wat`'s **type-parent** `Lru/*` joins, which must not move and did not.

| | mine |
|---|---|
| converted floor | **78 / 6014** |
| ⭐ FIXED / NEW vs the eighth draw | ⭐⭐ **29 / 0 — strict subset** |
| ⭐⭐ `UnresolvedReference` in the converted log | **40 → 0 — the whole class is GONE** |
| frozen unspellable list | **8 → 1** (`Fault/of`, a defmacro, rescued at the consult) |
| landable floor | **6014 / 6014 GREEN ×2** |

⭐ **The non-vacuity floor moved `slash_joined >= 16` → `>= 9`**, one below the measured 10 — the
nine type-parent `Lru/*` rows plus `Fault/of`, i.e. **the rows that make a green mean something.** It
still catches the population vanishing (converted stdlib → 1, measured). ⭐ **Re-aimed, not loosened,
and it says so.**

## ⛔⛔ PART 2 — MY PREMISE WAS WRONG, AND THE STOP IS CORRECT

I wrote: *"ONE interpolation template … mints the whole `<svc>/<op>` family."* ⛔ **Verified here:
27 interpolation sites in `wat/service.wat` carry a `/`.** `:2022` `{b}/{op-str}` is one;
`:2179` `{b}/stop` is a separate literal; `:1381`/`:2036`/`:2045` mint `{b}::{op}/Request` and
`/Response`. **The rider measured 96 minted names, 340 occurrences, 130 files — against my
"24 occurrences / 6 files", a ~17× understatement in both columns.**

⭐⭐ **AND THE `/` MINT IS LOAD-BEARING.** Verified here — the macro mints these with `::`:

```
{b}::init   {b}::serve   {b}::dispatch-admin   {b}::extract-addr
{b}::hibernate-project   {b}::service-forms    {b}::stop-project
```

⛔ **A user op named `init` or `serve` would collide with the macro's own internals under a `::`
mint.** The `/` join **keeps the user's op namespace disjoint from the macro's.** My brief proposed
changing it without knowing that. ⭐ **The rider found it with two probes — both `--check` rc 0 today —
and stopped.**

⭐ **And the scope finding that decides it:** a fully converted call to a minted name is **rc 0** —
the symbol call head already flips. **The minted family is NOT what blocks 8d-iii.** It lives on the
permissive door 255.8 wants closed; closing that door must precede any mint change.

## ⛔ The 416 variant accessors ARE in this class — and it stopped, as instructed

`(:u::Demo.Has/has d)` → `(u.Demo.Has/has d)` → rc 1, `:path ":u::Demo::Has::has"`. ⛔ **Worse than
the spawn class: `ns_to_wat_path` destroys the `.` as well as the `/`.** Untouched — a separate ruling.

## The brief was wrong five ways

1. ⛔⛔ **"ONE template"** → 21 non-type-parent mint sites (27 with `/` in total).
2. ⛔ **"24 / 6 files"** → 340 / 130 / 96 names.
3. ⛔⛔ **My non-vacuity row INVERTED its own control.** I wrote that `ProcessLaunch/bogus-field`
   *"is UNTOUCHED and still resolves."* ⛔ **It must NOT resolve — it is the deliberate unresolved
   reference.** A tree satisfying my row would have been broken.
   `[[feedback_an_expectation_row_must_be_satisfiable_by_a_correct_tree]]`. ⭐ The rider pinned
   what actually must hold: **refused, for its own reason.**
4. ⚠ "62 / 28 files" counted `src/` and replay fixtures; the codemod's list is **24**.
5. ⚠ Provenance line pointed at the weigh commit rather than the brief's own.

**Twenty-seven corrections across twenty-five stones.**

## What remains open — and it is now a short list

⭐ **Every `UnresolvedReference` is gone.** The converted floor's 78 are other classes
(`MalformedForm` 41, `CheckErrors` 19, `LociDiedError` 16, `DeclarationInExpressionPosition` 15,
`ProgramBodyEvalFailed` 14 — **all unchanged to the unit**, so this stone touched exactly its class).

**Needing the builder:**
1. ⛔ **The minted `<svc>/<op>` family** — 96 names, and the `/` is load-bearing against 7 reserved
   internals. **Not a respelling. A namespace-collision design question.**
2. ⛔ **`<b>::<op>/Request` / `/Response`** — lowercase `<op>` parent, same shape, only looked at by name.
3. ⛔ **The 416 variant accessors** — `ns_to_wat_path` destroys the `.`.
4. ⛔ **Nothing walls the class from returning outside the stdlib** — 255.4's lint is case-based and by
   construction cannot fire on a lowercase parent, which is exactly what made this stone legal.

## VERDICT

**PART 1 ACCEPTED. PART 2 STOP ACCEPTED.** clippy 0 (not cached, 11.73 s), census `no STOP-8`
213 = 213, delta **3 / RECOVERY 0**, `one_member_join` **run and green**, ledger 220 (unmoved, and
it cannot see `.wat`). Pushing.
