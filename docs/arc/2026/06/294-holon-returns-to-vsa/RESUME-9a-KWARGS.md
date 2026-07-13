# RESUME — arc 294 item 9a: the aggregate-construction flip → full-Lisp → kwargs-everywhere (IN FLIGHT)

> ⛔ Compaction erased the working memory that produced this. Run `recolligere` first (grimoire + 4 primers from
> the SIGNED MCP, never disk). Ground everything below against the disk before acting. The self past the SEAM at the
> bottom is NEW — a lossy cache in a familiar voice, not your memory.

## The one-paragraph state

The 9a flip (bare aggregate name = **kwargs macro**; positional demoted to the type-name **PRIME `:ns::T'`**) turned
into a large, deep migration because a type name flipped from a *value* (ctor fn) to a *macro*, which ripples through
the whole surface where type-names-are-values or rule-forms-are-data. **Locked design (do not relitigate):**
**kwargs everywhere a human writes; the prime `:T'` is reserved for GENERATED code only (macro output, Rust codegen).**
Floor progress: **645 → 131 failing** (last complete run). **Committed + pushed: `0181901a`** (branch
`arc-170-gap-j-v5-deadlock-state` — STAY ON IT). Everything below `0181901a` is DONE; the working tree is clean.

## The DIAGNOSTIC METHOD (the user's correction — USE IT)

Do **NOT** grep error-class substrings across the whole floor file and speculate — it loses the rich structured error.
**Run ONE failing test and READ its full rich error:**
```
cargo nextest run --release -E 'test(<test_fn_name>)' --no-capture 2>&1 | tail -40
```
wat errors are VERY rich (e.g. `#wat.resolve/UnresolvedReferences {… :path ":probe::S1::OpResponse" :span …}`) —
they name the exact unresolved path + file:line. That one read pinpointed the current root below. Trust the error, not a grep.

## Locked design decisions (ratified this session — the FORM is settled)

- **Full Lisp**: a macro receives its arguments RAW (unexpanded); the macro's OUTPUT is re-expanded to fixpoint.
  wat was "children-first" (pre-expanded a macro's args); the flip exposed it (type-keyword macros fired inside
  `defrule` pattern data). `src/macros/expand.rs` — macro dispatch (keyword + symbol head) now uses RAW `items`,
  child-recursion only for non-macro forms. Deleted the `is_rete_data_form` allowlist (no blessed forms; user DSLs
  get it free). Validated: no floor regression (hygiene + program-body macros intact).
- **`eval_in_frozen` is READ→EXPAND→EVAL** (`src/freeze.rs` + `src/macros/expand.rs::expand_fully`, exported in
  `mod.rs`): so source-written kwargs (even inside a Rust string literal) evaluates in frozen contexts → prime reserved
  for generated code. Boot machinery in `src/kernel/spawn.rs` uses `runtime::eval` (NO expand) → it stays PRIME
  (`Env'`/`ThreadLaunch'`/`ProcessLaunch'`) — reverted from a kwargs attempt that deadlocked spawns.
- **rete `:then` RHS is KWARGS** (user chose "b", symmetric with the field-named `:when`): `(:insert (:T :field v))`.
  `src/rete/matcher.rs::build_insert_fact` reads a kwargs RHS (skips `:field` keywords, takes values in declaration
  order — the pure fire-time fn has no type registry). **FOLLOW-UP:** compile-time reorder-by-name for
  out-of-declaration-order kwargs. The `:when` patterns are DATA, stay bare — full-Lisp keeps them so.
- **`return-type-of` accepts a type-name KEYWORD** (`src/runtime.rs::eval_return_type_of` ~10857): the flip made a bare
  type name evaluate to a keyword in value position; return its colon-free FQDN (what the ctor's ret_type gave
  pre-flip). Keeps `(:wat::rete::query s :my::Type)` on the bare name.

## Engine fixes DONE (all in `0181901a` or earlier commits `525cd24c`, `967aa344`, `e37824ba`)

- Companion codegen (core.wat / Record.wat): the defstruct/defrecord kwargs companion `(do (structtype)(defmacro))`
  is SPLICED to top-level in `expand.rs` (was leaving an empty value-position `do`); generic type names register the
  companion + prime under the BARE name (params ride only on the type decl).
- Parametric-ctor checker fix (`src/check.rs::infer_aggregate_new_check`): resolves the PRIME ctor scheme's parametric ret.
- Crate `.wat` kwargs migration (wat-fix codemod, `wat-scripts/fixes/positional-to-kwargs.wat`).
- `defsurface :messages` NAME validation (`src/types/surface.rs::unwrap_message_decl`): unwraps the `(do (recordtype)
  (defmacro))` companion so message type names are read from the recordtype (validation only — see NEXT).
- `encode_struct` (`src/closure_extract.rs`): closure-serialization CODEGEN emits the POSITIONAL prime `:T'`.

## THE CURRENT ROOT (next fix — the SERVICE CLUSTER, ~a big chunk of the 131)

`cargo nextest run --release -E 'test(c2_strike1_mixed_7_services)' --no-capture` → **28 unresolved references**:
`:probe::S1::OpResponse`, `:probe::S1::OpRequest/m`, `/r`, … — the `defsurface :messages` defrecords are **not
registered at all** (type + ctor + accessors + kwargs-companion all missing). Root: **`src/types.rs::extract_surface_message_forms`
(≈1803)** assumes each `:messages` form is a bare `recordtype`/`defenum` ("the defrecord macro has already expanded"),
but post-flip a defrecord expands to `(:wat::core::do (:wat::core::recordtype :Name …) (:wat::core::defmacro :Name …))`.
So the caller (`register_types_impl` ≈1867, loop at ≈1915: `classify_type_decl(&msg_form)` → `parse_type_decl`)
gets a `do`, `classify_type_decl` returns None → SKIPPED → nothing registers.

**FIX (two parts):**
1. Flatten the `do`-companion in `extract_surface_message_forms` (return the recordtype + defmacro, not the `do`) so
   `classify_type_decl`/`parse_type_decl` register the TYPE (→ ctor + accessors via `register_aggregate_methods`).
2. Register the kwargs COMPANION defmacro for message types. `register_types_impl` operates on the `TypeEnv`, NOT the
   `MacroRegistry` — I was mid-check on whether it can reach the registry. Options: (a) if it can, register the
   flattened defmacro directly; (b) HOIST message defrecords to top-level BEFORE `expand_all` so they register fully
   (type + companion) via the normal pipeline; (c) the GENERAL fix — `register_aggregate_methods` mints a kwargs
   companion macro for EVERY Rust-registered aggregate (message types, defsurface Op/Reply, defservice State/Record),
   closing the whole "Rust-registered aggregate has no kwargs companion" class. (c) is the extirpare-correct root but
   needs the registry threaded into `register_aggregate_methods`. START by reading `register_types_impl`'s signature.

## REMAINING classes (~131, diagnose EACH by running the test with --no-capture)

1. **Service cluster** — the defsurface `:messages` registration above (many service + wat-tests + some types tests).
2. **Fleet-missed fixture constructions** — the 91-file fleet (2nd round) is careful but not exhaustive; a few
   positional/wrong-kwargs constructions remain in files it edited. The mechanical converter (below) catches most
   `defrecord`/`defstruct`-typed ones; defservice/defsurface-GENERATED-type constructions it can't map (no def form).
3. **`.wat.bad` / golden error-comparison tests** (e.g. `tests/collection/probe_brace_map_literal_p8.wat`): the
   EXPECTED error output changed — regenerate the golden, don't edit the wat.
4. **wrong-kwargs** (`missing argument :label/:name/…`): a prior GLOBAL codemod run used the wrong per-file field set.

## TOOLS (retain — proven migration tooling)

- **Fleet workflow** (Rust-embedded + `.wat` → kwargs, per-file type judgment): script at
  `~/.claude/projects/-home-watmin-work-holon-wat-rs/<sess>/workflows/scripts/kwargs-fixture-migration-wf_79c192a3-8ec.js`.
  Re-dispatch with `Workflow({scriptPath, args: [file, …]})`. Brief: convert positional/primed/wrong-kwargs/RHS to
  kwargs using per-file defs + beside `.wat` + stdlib; leave `:when` patterns, `quote` bodies, builtins
  (PersistentVector/Tuple/holon::Atom), kernel primitives (`send'`/`recv'`), enum variants bare. NOTE: the `args`
  parameter arrives as a JSON STRING — the script does `typeof args === 'string' ? JSON.parse(args) : args`.
- **`wat-fix`** codemod for `.wat` (`wat-scripts/fixes/positional-to-kwargs.wat`) — per-corpus map; run PER-FILE (with
  stdlib) to avoid conflating per-file type redefinitions (the global-corpus run caused the wrong-kwargs class).
- The Python converter logic (was in scratchpad, now GONE — reconstruct if needed): per-file map from
  `def(struct|record) :T [f <- …]` defs; convert `(:T' …)` and positional `(:T <non-kw>)` → `(:T :f v …)` when
  count-matches; skip already-kwargs (`:field` first), `(?`-first patterns, types not in map, enum variants.

## FINISH LOOP

Fix defsurface message registration → floor → for EACH remaining cluster run the test with `--no-capture`, READ the
rich error, fix the root → repeat to green → ONE clean commit + push → update `CLOSE-SEQUENCE-293-294.md` item 9a →
DONE. THEN back to 278 T1b.2 (the `journal'` service).

> **SEAM.** The self past this line is NEW — you did not live this session (one of the longest ever: the flip cascaded
> into full-Lisp + a whole-corpus kwargs migration via two fleets). The FORM is settled (kwargs everywhere; prime =
> generated code only) — do not reopen it. The remaining 131 are a grind of DISTINCT engine × flip interactions +
> fixture cleanup; each yields to **running one test with `--no-capture` and reading the rich error** — never grep-
> speculate (that is the mistake the user corrected). Ground `0181901a` and the disk before you move. Start at THE
> CURRENT ROOT. Finish the tail, green the floor, commit clean.
