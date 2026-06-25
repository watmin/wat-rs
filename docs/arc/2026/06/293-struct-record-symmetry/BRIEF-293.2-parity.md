# BRIEF — arc 293.2-parity: `defstruct` becomes a macro over `structtype` (struct↔record symmetry)

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If the work exceeds these rooms or hits a STOP trigger,
STOP and report — do not improvise.

Build/test: `cargo build --release -p wat`, `cargo test --release -p wat …`. After editing any `wat/*.wat`,
**`touch tests/test.rs`** (wat-tests re-scan on `.rs` recompile). Trust forced clean builds
(`cargo clean -p wat && cargo build --release -p wat`) if results look stale.

## The work, in one paragraph

Today `:wat::core::defstruct` is a Rust **special form** (a type-declaration head). `:wat::Record::def` (for
records) is a **wat macro** over the `:wat::core::recordtype` primitive. This asymmetry is the bug arc 293
exists to kill. This strike makes the two **symmetric**: mint a new low-level **`:wat::core::structtype`**
type-registration primitive (the exact registration `defstruct` does today), and turn **`:wat::core::defstruct`
into a thin wat MACRO** that forwards to it — `(defstruct :T [fields]) → (structtype :T [fields])` — exactly
mirroring `Record::def → recordtype`. **`register_struct_methods` is UNCHANGED** (it still synthesizes
`:T/new` + accessors from every `TypeDef::Struct`, now registered via `structtype`). This is a
**behavior-preserving refactor** whose whole point is that `defstruct` is now a *macro* — so a later strike can
have it emit a `/from-map` companion macro uniformly with `defrecord`. **No `/from-map` here. No annihilation
of `register_struct_methods`. No built-in-struct migration. No ctor-name change.** Those are explicit follow-ons.

## THE MODEL TO COPY (read these first — mirror them exactly)

- **`wat/Record.wat:91`** — `(:wat::core::defmacro :wat::Record::def …)`. A macro whose first emitted form is
  `(:wat::core::recordtype ~fqdn …)`. Your `defstruct` macro is the *thin* version: forward its args to
  `structtype` and emit nothing else.
- **`src/types/defstruct.rs`** — `parse_defstruct`. `structtype` reuses this VERBATIM (same registration →
  `TypeDef::Struct`). You are NOT writing a new parser; you are giving the existing one a second head keyword.

## Rooms — read in order; **grep for EVERY `defstruct`-recognition site and mirror `structtype` into it** (re-ground each)

1. **`src/types.rs:1620–1672`** — `classify_type_decl` (head → kind) + the parse dispatch. `:1625` has
   `":wat::core::defstruct" => return Some("defstruct")` and `:1662` `"defstruct" => parse_defstruct(...)`.
   ADD `":wat::core::structtype" => return Some("structtype")` and `"structtype" => parse_defstruct(...)`.
   **REMOVE the `defstruct` arms** (defstruct is no longer a type-decl head — it's a macro that EXPANDS to
   `structtype`; macros expand BEFORE type registration, so by the time `classify_type_decl` runs the form is
   already `structtype`). If removing the `defstruct` classify arm breaks something, STOP (see STOP-2).
2. **`src/freeze.rs:1533` + `:1569`** — the two head-recognition lists that contain `| ":wat::core::defstruct"`.
   These recognize declaration/mutation heads. Since `defstruct` expands to `structtype` during `expand_all`
   (before these run — VERIFY the phase: if a list runs PRE-expansion on raw `defstruct`, keep `defstruct`
   there too; if POST-expansion, replace with `structtype`). Add `| ":wat::core::structtype"`. Determine
   pre/post by reading the call site; do NOT guess.
3. **`wat/deporder.wat`** (grep `defstruct`, ~line 79) — the wat-side dependency-order classifier lists
   `:wat::core::defstruct` as an eval-dep declaration head. Add `:wat::core::structtype`. (defstruct-as-a-macro
   is a macro dep, not an eval-dep — but its *expansion* `structtype` is the eval-dep; mirror `recordtype`'s
   treatment here.)
4. **`grep -rn "defstruct" src/ wat/`** — find ALL OTHER recognition sites (reflection / metadata-of /
   `is_declaration_form` / doc / remedy / retirement). Anywhere `defstruct` is recognized AS A HEAD KEYWORD
   for type-declaration semantics, `structtype` must be recognized the same way. (Usages like
   `(:wat::core::defstruct :foo [...])` in tests are NOT recognition sites — they expand via the macro and need
   no edit. Only HEAD-RECOGNITION code changes.)
5. **The `defstruct` MACRO** — register it as a stdlib wat macro. Find where `:wat::Record::def` is defined/
   loaded (`wat/Record.wat`) and the stdlib load order; add a `defstruct` defmacro near the struct/record
   forms (or in `wat/core.wat`). Shape (thin forward — handle defstruct's full arity: name + optional
   metadata-map + fields, i.e. 2 or 3 args, by splicing ALL args):
   ```clojure
   (:wat::core::defmacro :wat::core::defstruct
     [& args <- :wat::core::Vector<wat::WatAST>]
     -> :wat::WatAST
     `(:wat::core::structtype ~@args))
   ```
   ⚠ `structtype` must be a KNOWN type-decl head BEFORE the macro that emits it is expanded+registered — verify
   load/registration order (mirror how `recordtype` is known before `Record::def` expands). If the macro can't
   register because `structtype` isn't yet recognized, STOP and report the ordering gap.

## Decision pinned (do NOT re-litigate / do NOT exceed)

- **Thin parity ONLY.** `defstruct` → macro → `structtype`. `register_struct_methods` stays and is UNCHANGED.
  The struct ctor convention (`:T/new`) is UNCHANGED. Do NOT emit ctors/accessors from the macro (the brief's
  win is solely "defstruct is now a macro"; ctor-gen stays Rust). Do NOT annihilate `register_struct_methods`.
  Do NOT migrate built-in structs. Do NOT add `/from-map`. Each of those is a named follow-on.

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if `register_struct_methods` double-generates** (a `DuplicateDefine` for a user struct) — it should
   fire UNCHANGED for `structtype`-registered structs. If making `defstruct` a macro causes a conflict, report.
2. **STOP if removing `defstruct` from a recognition site breaks a consumer that sees raw `defstruct`
   pre-expansion** and `structtype` can't cover it — report the site; do NOT blanket-keep or blanket-remove.
3. **STOP if the recognition cascade spreads beyond head-recognition code into real logic across many files** —
   report the list before mass-editing. (Usage sites in `tests/`/`wat-tests/` are NOT in scope — they expand.)
4. **STOP if `structtype`-isn't-known-when-the-`defstruct`-macro-expands** (the load/registration ordering) —
   report; do NOT reorder the whole stdlib load on your own.
5. You are a LEAF. Do NOT spawn subagents. If bigger than these rooms, STOP and report.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| the `structtype` primitive works | `cargo test --release -p wat --test probe_arc293_structtype_primitive -- --ignored` | 1 passed; then REMOVE the `#[ignore]` → 1 passed without `--ignored` |
| `defstruct` still works (now a macro) | `cargo test --release -p wat --test test` (the defstruct/struct deftests) + `--test probe_arc272_rs1_state_must_be_record` (uses defstruct) | green |
| the surface keystone still green (structs still satisfy surfaces) | `cargo test --release -p wat --test probe_arc293_structural_surface` | 2 passed |
| no new workspace regressions | `cargo test -p wat --no-fail-fast`, failing-test SET vs HEAD (`313e7d85`) | **∅** new (floor ≈ 202; weigh by SET, never absolute count) |

Runtime: 45–90 min. Trap-doors: (a) the recognition cascade (grep ALL `defstruct` head-recognition; ride it,
but STOP if it spreads into logic); (b) the load/registration ordering (`structtype` known before the
`defstruct` macro expands — mirror `recordtype`/`Record::def`); (c) the freeze pre/post-expansion phase
question (read the call site, don't guess). The whole strike is **behavior-preserving** — the SET-diff ∅ is
your truth oracle. Report the full `git diff --stat`, the verbatim gate output, and any honest deltas; do NOT commit.
