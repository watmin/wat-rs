# Arc 028 — load/eval iface drop + hoist to root

**Status:** opened 2026-04-23. Side quest from arc 027 which
itself was a side quest from lab arc 002's not-yet-opened
deftest-ergonomics concern. The pattern holds: each cave-quest
resolves a gap the previous one couldn't have named without
walking closer.

**Builder direction:**

> i think we should also remove file-path as an arg for load...
> we'll /maybe/ have a network load or something later.. we left
> comments on http, s3, github.... /if/ we bring those.. they
> be named load! commands like signed and digest evals....
>
> let's remove the file-path noise and make the argument itself
> a file path (it was always that...)
>
> oh... nvm... we have load by string.. interesting.... ------
> hrm.... we should have... (load! "/path/to/file.wat") and
> (load-string! "...") instead of a param for it....
>
> ah.. interesting... i think... we do :wat::load... :wat::eval...
> now - that's a good pattern - if that's yet another side
> quest / cave / arc - so be it..

Two renames, one arc:

1. **Drop the interface keywords.** `:wat::load::file-path`,
   `:wat::load::string`, `:wat::eval::file-path`,
   `:wat::eval::string` retire. Each form takes its source
   directly (path or string) as the first argument. Future
   network variants (`load-http!`, `load-s3!`, `load-github!`)
   land as named siblings, matching the digest/signed pattern
   already established.

2. **Hoist to root.** The nine load/eval forms move from
   `:wat::core::*` to `:wat::*` directly. Language-substrate
   forms sit next to `:wat::WatAST` at the namespace root.
   Matches how Scheme, Clojure, Python, JS place `load`/`eval`/
   `require`/`import` at the top of their identifier trees, not
   nested under a `core` sub-namespace.

---

## Surface before/after

### `load` family

```scheme
;; BEFORE (today):
(:wat::core::load!        :wat::load::file-path "path/to/file.wat")
(:wat::core::load!        :wat::load::string    "(source text)")
(:wat::digest-load! :wat::load::file-path "path"
                          :wat::verify::digest-sha256
                          :wat::verify::string "<hex>")
(:wat::signed-load! :wat::load::file-path "path"
                          :wat::verify::signed-ed25519
                          :wat::verify::string "<sig>"
                          :wat::verify::string "<pk>")

;; AFTER (arc 028):
(:wat::load!         "path/to/file.wat")
(:wat::load-string!  "(source text)")
(:wat::digest-load!  "path"
                     :wat::verify::digest-sha256
                     :wat::verify::string "<hex>")
(:wat::signed-load!  "path"
                     :wat::verify::signed-ed25519
                     :wat::verify::string "<sig>"
                     :wat::verify::string "<pk>")
```

### `eval` family

```scheme
;; BEFORE (today):
(:wat::eval-ast!    <ast-value>)
(:wat::eval-edn!    :wat::eval::string    "(source text)")
(:wat::eval-edn!    :wat::eval::file-path "path")
(:wat::eval-digest! :wat::eval::file-path "path"
                          :wat::verify::digest-sha256
                          :wat::verify::string "<hex>")
(:wat::eval-signed! :wat::eval::file-path "path"
                          :wat::verify::signed-ed25519
                          :wat::verify::string "<sig>"
                          :wat::verify::string "<pk>")

;; AFTER (arc 028):
(:wat::eval-ast!     <ast-value>)
(:wat::eval-edn!     "(source text)")              ;; EDN source
(:wat::eval-file!    "path")                       ;; new — split
(:wat::eval-digest!  "path"
                     :wat::verify::digest-sha256
                     :wat::verify::string "<hex>")
(:wat::eval-signed!  "path"
                     :wat::verify::signed-ed25519
                     :wat::verify::string "<sig>"
                     :wat::verify::string "<pk>")
```

### What retires

- `:wat::load::*` namespace — all four keywords (`file-path`,
  `string`, `http-path`, `s3-path`) gone. The namespace is empty
  and unused.
- `:wat::eval::*` namespace — same four shapes gone.

### What stays

- `:wat::verify::*` keywords — two payload-location shapes
  (`string` for inline hex/base64, `file-path` for sidecar). The
  verify payload is ONE of several genuinely-different locations,
  so keyword-based dispatch stays honest (each call site picks
  the shape it has). Plus the algo keywords (`digest-sha256`,
  `signed-ed25519`, etc.) — those name the integrity
  CONTRACT, not the location.
- Everything else under `:wat::core::*` — `define`, `let*`, `if`,
  `match`, `try`, `quote`, `forms`, `defmacro`, `vec`, `HashMap`,
  `HashSet`, `get`, `assoc`, `conj`, `contains?`, arithmetic,
  primitive-type operations. These are **language vocabulary**
  — how authors express computation. They stay scoped under
  `core` because they're internal to the language, not substrate
  interfaces to the world.

---

## The two-tier namespace shape that emerges

After arc 028 ships, a clean split:

| Tier | Prefix | What lives here |
|---|---|---|
| **Substrate** | `:wat::<form>` | Forms that interface wat with the outside world: load!, load-string!, digest-load!, signed-load!, eval-ast!, eval-edn!, eval-file!, eval-digest!, eval-signed!. Plus the AST type `:wat::WatAST`. |
| **Vocabulary** | `:wat::core::*` | Forms that express computation inside wat: control flow (define, let*, if, match, try, cond), collection constructors/accessors, arithmetic, primitive-type operations. |

Other top-level namespaces stay as they are:
- `:wat::holon::*` — holon algebra (HolonAST + primitives + measurements + 10 wat-written idioms)
- `:wat::kernel::*` — CSP primitives (spawn, send, recv, select, queue, fork, signals)
- `:wat::io::*` — stdio substrate (IOReader/IOWriter, println)
- `:wat::std::*` — the stdlib BUILT on primitives (stream combinators, test harness, services)
- `:wat::config::*` — runtime-committed configuration
- `:wat::verify::*` — verification payload + algo keywords (kept — multi-shape dispatch)
- `:wat::test::*` — test primitives (deftest, assert-*, run, etc.)
- `:user::*`, `:rust::*` — user + Rust-interop surfaces (unchanged)

---

## Slices

### Slice 1 — rename the four existing load primitives

Target: `src/load.rs`.

- **Drop interface keyword on plain `load!`.** First arg is now
  the path (or a source string — see slice 2). Arity drops from
  2 to 1.
- **Drop interface keyword on `digest-load!`**, `signed-load!`.
  Arity drops from 5→4, 7→6. Remaining args stay: path, then
  verify keywords + payloads.
- **Update `match_load_form`** (the parser-dispatch helper) to
  accept the new shapes.
- **Delete `:wat::load::*` keyword recognizer** in `resolve.rs`
  (the reserved-prefix check), if any.
- Rust unit tests update — every call-site string in tests gets
  the new shape.

### Slice 2 — split load! into load!/load-string!

Target: `src/load.rs` + `src/check.rs` + `wat-macros/` (if macro
recognition is involved).

- `(:wat::load!)` — ONE arg (path). File-only.
- `(:wat::load-string!)` — ONE arg (source). String-only.
- Parser dispatch chooses based on form head, not first-arg type.
- The `SourceInterface::String` branch of today's `load!`
  becomes `load-string!`'s only branch.

### Slice 3 — rename the four eval primitives + split eval-edn!

Target: `src/runtime.rs`.

- `eval-edn!` takes ONE string arg (source text only). Drops
  the interface keyword.
- `eval-file!` — NEW form. Takes ONE path arg. Reads the file
  (via the outer loader) and evaluates like eval-edn!.
- `eval-digest!` drops the interface keyword. Arity 5→4.
- `eval-signed!` drops the interface keyword. Arity 7→6.
- `eval-ast!` unchanged (already takes one AST value arg).
- Rust unit tests updated per call-site.

### Slice 4 — hoist all nine forms from `:wat::core::*` to `:wat::*`

Target: `src/runtime.rs` dispatch table + `src/check.rs` scheme
registrations + every call site in wat source.

- Evaluator dispatch: change the match arms from
  `":wat::core::load!"` to `":wat::load!"` etc.
- Type-check scheme registrations: same.
- `src/check.rs` reserved-prefix gate: `:wat::load!` etc. become
  reserved under `:wat::*` directly (already covered — `:wat::*`
  is the full reserved-prefix root).
- **Call-site migration.** Every `:wat::core::load!` in:
  - `wat/std/*.wat` — wat-rs stdlib
  - `wat-tests/*.wat` — wat-rs test suite
  - `src/*.rs` — any inline wat source strings in Rust
  - Every arc's DESIGN + INSCRIPTION docs that quote the forms
  - `docs/USER-GUIDE.md`, `docs/CONVENTIONS.md`
  - `holon-lab-trading/wat/*.wat`, `holon-lab-trading/wat-tests/*.wat`
  - `holon-lab-trading/docs/proposals/.../FOUNDATION.md` + all
    058 sub-proposals
  - `holon-lab-trading/BOOK.md` (every chapter quoting load/eval)
  - Every arc INSCRIPTION in wat-rs that quotes these forms

Migration is mechanical — sed across the workspace. Every `load!`
usage changes from `:wat::core::load!` to `:wat::load!` etc.

### Slice 5 — migrate old test-call shapes to new

A few tests today have literal wat source with old-shape
`load!` / `eval-*!` calls. These need rewriting to new-shape.
Already-ported tests in slices 1 + 2 + 3 get the shape updates
alongside their arity changes; slice 5 catches any remaining.

### Slice 6 — INSCRIPTION + doc sweep

- `docs/arc/2026/04/028-load-eval-rename/INSCRIPTION.md`
- `docs/CONVENTIONS.md` — namespace table updated: two-tier
  split documented, retired namespaces noted, verify tier
  documented.
- `docs/USER-GUIDE.md` — every call-site example updated. New
  section on the two-tier split (substrate vs vocabulary).
- `docs/README.md` — arc tree.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  rows for all nine primitives updated.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — new row.
- 058 proposal docs — any that quote old-shape forms, sweep.

---

## Tests

Existing tests survive the rename (just change the wat source
strings). No new primitives, so no new test classes — unless
slice 3's `eval-file!` is net new (it is), in which case a
couple of Rust unit tests cover it:

- `eval_file_reads_source_and_evaluates`
- `eval_file_unknown_path_errs_as_evalerror`
- `eval_file_mutation_form_refused`

---

## Doc sweep (meta-concern)

The migration touches EVERY place that quotes a load/eval form.
That's a lot. Sweep ordering:

1. Rust source + wat source + wat-tests — get tests green first.
2. wat-rs docs (USER-GUIDE, CONVENTIONS, arc INSCRIPTIONs).
3. Lab repo (wat/, wat-tests/, docs/proposals/, BOOK.md).

Each commit stays focused on its own target area. The arc ships
in 3-4 commits total, each rebuildable.

## What this arc does NOT ship

- Arc 027's remaining slices (:None scope inherit, wat::test!
  scope widen, lab test migration). Those resume after arc 028.
- Hoisting other forms from `:wat::core::*` to root. `define`,
  `let*`, `if`, `match`, etc. stay in `core`. They're language
  vocabulary, not substrate. Future arc can revisit if a clear
  separation reason surfaces.
- Network-variant forms (`load-http!`, `load-s3!`,
  `load-github!`, `eval-http!`, etc.). Shipped when a real
  caller demands; arc 028 just reserves the naming shape.

## Why this is inscription-class

The surface was already under the builder's eye during arc 027
planning. Naming cleanup emerged naturally from the TypeScript-
stance direction. Shipping both the iface-drop AND the hoist in
one arc halves the call-site migration (one rewrite per site,
not two). Same shape as every arc from 019 onward — code-led,
spec-follows, clean commits.
