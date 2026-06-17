# Arc 275 — the stdlib load order, verified by a self-hosted algorithm (`deporder`)

> **STATUS: STRIKE-READY (275.1).** Opened 2026-06-17. A self-hosted analysis tool that derives the
> stdlib's true dependency order from the source and **verifies** the load order respects it — the
> enforcement is an algorithm, not a hand-maintained declaration.

## The trigger

Grounding 260.3's home, the orchestrator noticed `core.wat` sits at **position 27 of 32** in
`STDLIB_FILES` (`src/stdlib.rs:30`) — after `test.wat`, `edn.wat`, every holon module, the whole kernel
subsystem. Builder: *"we never chose the order… just put core first… because it's core?"* then, on
watching the orchestrator hand-grep the dependency graph: *"whoa — can you just write a wat tool that
does this for us?"* and *"it IS the check, once we write it correct — it's just an algorithm."*

The order is **accreted, not chosen.** The fix is not a tasteful reshuffle a human re-checks; it is an
**algorithm** that computes the dependency truth and makes a violation a red build. Self-hosted: wat
analyzing wat.

## Why an algorithm beats a declared phase enum (the design pivot)

The first cut of this arc was a hand-assigned `StdlibPhase` enum + a monotonicity test. **Rejected.** A
human declaring each file's layer is a convention that can be *wrong* (mis-assign a phase, the test
passes, the lie ships). The extirpare top rung is to **compute** the layering from actual references so
it *cannot* be wrong. The `deporder` tool reads the source, classifies every cross-file reference, and
derives the real dependency DAG — the check is deterministic and self-correcting. No enum; no hand
phases.

## Grounded facts (crawled this session, cited)

- **Defmacro visibility is order-FREE.** `register_stdlib_defmacros` (`src/macros/parse.rs:29`) walks
  the whole concatenated stdlib in one pre-expansion pass — every `defmacro` registers before any
  expansion (`src/stdlib.rs:233-236`). So a reference to a defmacro imposes **no** load-order
  constraint. (This is why `core.wat` can reference `:wat::Record::def`, a defmacro at `Record.wat:91`,
  though `Record.wat` loads earlier.)
- **What order DOES carry:** *eval-time* deps — a file's runtime-evaluated `defn`s / `defenum`s and
  substrate dispatch may only call things registered **before** it (documented at `stdlib.rs:241-243`,
  `:248-251`: string/list "load after core").
- **The introspection primitives exist and compose** (proven by `fix.wat`, which already walks wat AST):
  - `:wat::io::read-file` (String→String), `:wat::io::list-dir` (String→Vector<String>)
  - `:wat::core::read-string` (source→forms)
  - `:wat::core::ast-kind` (→ "list"/"keyword"/"symbol"/…), `:wat::core::ast-name` (→ the name string),
    `:wat::core::ast->children`, `:wat::core::first`/`rest`/`drop`/`take` (`fix.wat:24-91` worked ref)
  - `ast-span` carries only `{:line :col}` (`check.rs:16893`) — **no file path**, so per-file
    attribution comes from reading each file individually (not from the global `forms`).
    **Design vindication (builder, 2026-06-17):** an earlier call deliberately kept `ast-span` free of
    a filename as *redundant* — and `deporder` proves it right. The tool reads file-by-file, so each
    form's file is the loop variable; a span-filename would copy that one fact onto every node inside an
    already-known file. Attribution belongs to the reader (the iteration), not the node. The minimal
    span was the correct primitive.
- **The load order lives in Rust** (`STDLIB_FILES`), unreachable from wat by reflection. So `deporder`
  takes the ordered path list as an **input** (a pure function of it), and a test feeds it the real
  `STDLIB_FILES` order.

## The tool — `deporder` (named by an intueri cast)

- **File:** `wat/deporder.wat`  **Namespace:** `:wat::deporder::`  (English register, matching
  `fix.wat`'s precedent as a domain-noun-named wat dev tool — the discipline/Latin register is for
  spells, not callable namespaces.)
- **`:wat::deporder::analyze (paths: Vector<String>) → <report>`** — reads + parses each file, returns
  the per-file dependency classification + the DAG.
- **`:wat::deporder::verify (paths: Vector<String>) → <:ok | violations>`** — treats `paths` as the
  load order and asserts it respects every eval-dep; returns ok or the exact violations
  (`file X at pos i eval-depends on Y at pos j>i`).

### The algorithm

1. **Read + parse** each path: `read-file` → `read-string` → forms.
2. **Pass 1 — symbol map.** Walk each file's top-level forms; a form is a *definition* when its head
   ast-name is one of `:wat::core::{defn,defmacro,defenum,defalias,defprotocol,def,defclause,extend-type}`;
   the defined symbol is the next child's ast-name; the *kind* is the head. Build
   `symbol → (file, kind)`.
3. **Pass 2 — classify references.** Walk every form recursively; collect qualified keyword nodes
   (`ast-kind == "keyword"` && name contains `::` && starts `:wat::` — the `fix.wat:57-58` predicate).
   For each reference defined in *another* file:
   - kind `defmacro` → **order-free** (ignore)
   - kind `defn`/`defenum`/value → **eval-dep** (referencer must load after definer)
   - defined in **no** file → **intrinsic** (Rust, always-available; order-free)
4. **Build the DAG** (file → files it eval-depends on); `verify` checks the input order respects it;
   `analyze` returns the full classification.

### Contract decision (the one pinned interface choice)

`deporder` is a **pure function of the ordered path list** — `verify(paths) → ok | violations`. It does
**not** reach into Rust for the order (kept out of the tool; supplied by the caller), and it reads
sources from disk via `read-file` (the on-disk source == the baked source in a built tree). This keeps
the tool a deterministic algorithm with one input and one verdict.

## Decomposition

- **275.1 — build `deporder`** (this strike). `wat/deporder.wat` + `analyze`/`verify` + its own
  `deftest` suite (complectens: it carries its own proof — a tiny fixture where A defn-depends on B,
  plus a real-stdlib smoke). Register `wat/deporder.wat` in `STDLIB_FILES`. Then **run it on the real
  stdlib** and capture the dep DAG — that output informs 275.2.
- **275.2 — enforce + meaningful reorder.** Wire `deporder::verify` over the real `STDLIB_FILES` order
  as a **test** (red build on any eval-dep violation — the enforcement rung). Then reorder
  `STDLIB_FILES` into a meaningful foundational→derived order that `verify` confirms valid (core first),
  with a one-line *why-here* per entry + a doctrine comment that sits in **intueri's** lane (a vigilia
  on `stdlib.rs` observes the rule). *Open impl choice for 275.2: feed the order to `verify` via a Rust
  test extracting `stdlib_files()` paths (zero new surface) vs. a thin `:wat::stdlib::ordered-paths`
  intrinsic enabling a pure-wat deftest — decide when 275.2 opens.*

## Out of scope (rejected, not deferred)

- **No `StdlibPhase` enum / hand-assigned phases.** The algorithm computes the layering; humans don't
  declare it. (The pivot above.)
- **No change to the loading mechanism** — the single defmacro pre-pass stays exactly as is.
- **No new eval-order dependencies introduced.** If `deporder` reveals a real violation in the current
  order, that is *data* (extirpare) — fix the order, never add a workaround.
- **No re-homing of definitions** — `core.wat` is already honestly homed (all 16 defs are `:wat::core::`,
  grounded).

## The four questions

- **Obvious?** YES — `:wat::deporder::verify` says "verify the dependency order"; the violation report
  names the exact offending pair. `fix.wat` is the legible precedent for the shape.
- **Simple?** YES — one tool, one algorithm: read → classify references → DAG → check order. One concept
  (the order respects computed eval-deps).
- **Honest?** YES — the check is *computed from the source*, not declared, so it cannot lie; an
  order-free defmacro ref is correctly excluded; an unknown symbol is correctly an intrinsic. The
  enforcement test makes a violation un-shippable.
- **Good UX?** YES — adding a stdlib file that breaks the order fails the build with the precise dep;
  `analyze` gives a human the whole graph on demand. Self-hosted, so it grows with the language.
