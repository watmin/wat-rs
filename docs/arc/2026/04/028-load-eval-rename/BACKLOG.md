# Arc 028 — load/eval rename — BACKLOG

Six slices, mechanical after slice 1 locks the resolver shape.

- **ready** — can be written now.
- **obvious in shape** — follows once the prior slice lands.
- **foggy** — needs prior discovery.

---

## Slice 1 — rename the four load primitives (iface drop)

**Status: ready.**

Target: `src/load.rs`.

- `load!` arity 2 → 1. First arg IS the path (or string; split
  in slice 2).
- `digest-load!` arity 5 → 4.
- `signed-load!` arity 7 → 6.
- `match_load_form` helper + parse_load / parse_digest_load /
  parse_signed_load — accept new arity + shape.
- Delete any recognizer code for `:wat::load::*` namespace
  keywords.
- Rust unit tests for the three primitives — update every call-
  site wat string. Tests already exist; shape change only.

**Sub-fogs:**

- **1a — existing test count.** How many tests hold old-shape
  wat strings? Estimate ~15 in load.rs, more in resolver /
  check. Grep confirms before write.
- **1b — parse_load's current arity check.** Hard-codes
  "takes exactly 2 arguments." Flip to 1, update error text.

## Slice 2 — split load! into load!/load-string!

**Status: obvious in shape** (once slice 1 locks the resolver
for path-only).

Target: `src/load.rs`.

- `load!` stays as file-path-only (slice 1's shape).
- `load-string!` — new form, single arg (string source).
  Internally dispatches to the same pipeline load! uses via
  `SourceInterface::String`.
- Parser/dispatch: recognize the new form head. Same `match_load_form`
  entry, dispatch by head name.
- Rust unit tests — add `(load-string! "...")` variants.

**Sub-fogs:**

- **2a — SourceInterface enum.** Today's enum is
  `{ String(String), FilePath(String) }`. Split internally stays
  same (the resolver still dispatches by interface); the
  external-facing form names just expose the two variants
  separately.

## Slice 3 — rename the four eval primitives + split eval-edn!

**Status: obvious in shape** (once slices 1-2 land — the pattern
carries over).

Target: `src/runtime.rs`.

- `eval-ast!` unchanged (already 1-arg AST).
- `eval-edn!` — now 1-arg string only (drops the interface
  keyword). Was 2-arg with string-or-file dispatch; becomes
  string-only.
- `eval-file!` — NEW. 1-arg path. Reads the file via the
  outer loader, parses + evaluates. Internally uses
  `parse_and_run` on the loaded source.
- `eval-digest!` — 5→4 (drop interface keyword).
- `eval-signed!` — 7→6.
- Check-layer schemes updated.

**Sub-fogs:**

- **3a — `eval-file!`'s loader access.** Runtime-layer loader
  access at arg-eval time. Today's `eval-edn!` with file-path
  uses `resolve_eval_source` which calls `sym.source_loader()`.
  `eval-file!` does the same. No new primitive; just a split
  of the existing one.

## Slice 4 — hoist nine forms from :wat::core::* to :wat::*

**Status: obvious in shape** (after slices 1–3; this is the
namespace move).

Target: dispatch + scheme registrations + call-site sweep.

- `src/runtime.rs` dispatch table — change match arms from
  `":wat::load-file!"` → `":wat::load!"` for all nine forms.
- `src/check.rs` — scheme registrations (9 entries updated).
- `src/check.rs` reserved-prefix check — `:wat::*` already
  reserved, so new names are already protected. No new
  reservation needed.
- **Global call-site sweep.** Grep-and-replace across:
  - `src/` (Rust source — inline wat source strings in tests,
    error messages)
  - `wat/` (wat-rs stdlib)
  - `wat-tests/` (wat-rs test suite)
  - `examples/*/` + `crates/*/`
  - `docs/` (USER-GUIDE, CONVENTIONS, every arc's DESIGN +
    INSCRIPTION that quotes these forms)
  - `holon-lab-trading/wat/`
  - `holon-lab-trading/wat-tests/`
  - `holon-lab-trading/docs/proposals/` (058 FOUNDATION + all
    sub-proposals)
  - `holon-lab-trading/BOOK.md` (every chapter quoting
    load/eval — Chapter 16 onward has many)

**Sub-fogs:**

- **4a — sed pattern.** Literal `:wat::load-file!` →
  `:wat::load!`, same pattern for the other eight. No regex
  complexity; mechanical substitution. Grep first to verify
  count. Probably ~500-1000 occurrences total across both
  repos.
- **4b — historical quotes.** arc 007/011/012/016/019 etc.
  have INSCRIPTION.md files that quote old-shape forms. We
  sweep them — the INSCRIPTION records the SHIPPED contract,
  which at arc 028's time becomes the new-shape. This is
  honest bookkeeping.

## Slice 5 — migrate remaining call-site tests

**Status: obvious in shape** (should mostly fall out of slice 4).

Any tests slice 4's sweep missed (likely zero after careful
grep, but defensive). Rust tests + wat-level tests rerun; any
remaining failures get specific edits.

## Slice 6 — INSCRIPTION + doc sweep

**Status: obvious in shape.**

- `docs/arc/2026/04/028-load-eval-rename/INSCRIPTION.md`
- `docs/CONVENTIONS.md` — new subsection documenting the
  two-tier split (substrate at root vs vocabulary under core).
  The retired namespaces noted.
- `docs/USER-GUIDE.md` — full rewrite of the load / eval
  sections; two-tier split explained.
- `docs/README.md` — arc tree updated.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  all nine rows updated to new paths.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — arc 028 row.
- 058 FOUNDATION.md — namespace table at the top, if it
  documents the iface keywords, gets corrections.

---

## Working notes (updated as slices land)

- Opened 2026-04-23 — side quest from arc 027. Builder
  direction: "i think we should also remove file-path as an
  arg for load... let's remove the file-path noise and make
  the argument itself a file path (it was always that...)"
  plus the hoist: "we do :wat::load... :wat::eval... now -
  that's a good pattern."
- Arc 027 paused after slice 1 (dedup landed). Arc 027 slices
  2–5 resume after arc 028 so the lab test migration uses the
  new naming.
- **Closed 2026-04-23 same-day as opened.** Slices shipped:
  - `beade60` + `920c120` — slices 1+3 (iface drop + family split
    + string variants + signature-guard promotion).
  - `aa5bc9f` (lab) — lab migration for slices 1+3.
  - `8e9a40d` — slice 4 (hoist to `:wat::*` root + reserved-prefix
    collapsed to `:wat::` + `:rust::`).
  - `49ab0b6` (lab) — lab migration for slice 4.
  - INSCRIPTION + doc sweep commit (this final one).
- Slice 5 (straggler cleanup) folded into slices 1+3 and 4 as
  they landed — caught each integration-test site needing a
  different rename pattern. Nothing left standalone for slice 5.
- Final scope broader than initial DESIGN — arc added the
  string-verified variants (digest-load-string!, signed-load-string!,
  eval-digest-string!, eval-signed-string!, and the two
  eval-*-string-coincident? siblings) because existing tests
  exercised inline-source verification and that's a genuine
  deployment shape (builder's HTTP-Tokio handler scenario).
