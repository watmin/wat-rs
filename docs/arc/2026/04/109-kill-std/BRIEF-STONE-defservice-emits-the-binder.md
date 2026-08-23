# BRIEF — nothing MINTS and nothing RENDERS the angle form

Position 4 of `:-` went live in `69933d362`, so a macro can finally emit what it always should have.
You will spend that: stop `wat/service.wat` minting angle names, delete the branches that only existed
because two spellings did, and — the part no earlier census saw — **stop the substrate RENDERING the
retired spelling back at the user.**

Read `DESIGN-STONE-defservice-emits-the-binder.md` first. The tree is CLEAN and the floor is green at
4903/4903; keep it that way. Copy the report shape of `SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## STEP 1 — the renderers. Do this first; step 2 depends on it.

Four sites re-serialize a type back INTO `Head<A,B>`:

```
src/check.rs:16278      format!("{}<{}>", head, inner.join(","))
src/runtime.rs:13401    format!("{}<{}>", n, f.type_params.join(","))
src/runtime.rs:13480    format!("{}<{}>", name, scheme.type_params.join(","))
src/runtime.rs:13647    format!("{}<{}>", base, type_params.join(","))
```

They are USER-FACING. Measured on the current build:

```
:u::want: parameter #1 expects :wat::core::Vector<wat::core::i64>; got :wat::core::String
```

**A user who copies that type into their source gets a lex error.** Make them emit the surviving form:

```
type_params EMPTY  →  Head                    ← never `Head<>`
otherwise          →  (Head :- [A B])
```

**One shared helper, not four edits.** Four `format!`s with one shape is precisely the defect this arc
has spent the day removing; its home is beside the existing name/param-spec doors and it is subject to
the `one_param_spec` rune.

⚠ Expect golden `.edn` churn. **The ruling is KEEP PINNING THE SPAN and recapture** — see
`docs/arc/2026/06/296-diagnostics-fully-edn/BRIEF-296-WaveB1-complete-the-26.md` and the comment in
`tests/types/probe_arc293_W2b_enum_purity.rs`. Do not drop or normalise a field to avoid churn; the
pin discriminates the emitter. Verify each recapture is the same call site, only moved.

## STEP 2 — `(Head :- [])` ≡ `Head` at REFERENCE position

Measured broken today, and it falls out of step 1 once nothing emits `Head<>`:

```clojure
(:wat::core::defn :u::takes [x <- (:u::Plain :- [])] -> :i64 …)
  →  ":u::takes: parameter #1 expects :u::Plain<>; got :u::Plain"     ⛔
```

The rule already holds at declaration and at constructor — verify all three positions agree when you
are done. This is what unblocks step 3.

## STEP 3 — `wat/service.wat` stops minting and loses its branches

```
942-943   proto-op-ty-kw / proto-reply-ty-kw    DEAD BINDINGS. One occurrence each in the whole
                                                 file — defined, never used. They are the FIRST
                                                 thing the minting wall screams at, and they are
                                                 dead code. Delete them.
2374-85   launch-head-kw                         `wat::spawn::Locus/launch<A,B,C,D,E>` becomes the
                                                 BARE keyword, with `:- [...]` emitted as SIBLINGS
                                                 at the call site. That is position 4 and it is live.
500       proto-tp                               the `<…>` suffix string — dies with its last consumer.
1021 1024 1360 2014 2025                         five `(if (empty? proto-args) …)` branches become
                                                 UNCONDITIONAL. The macro always emits `:- [args]`.
```

★ **The exemplar is in the same file.** `proto-op-ty-ann` at line 1021 already mints the reference FORM
structurally off `proto-args` — ③ wrote it correctly. Every other site copies that shape, and the only
edit to the exemplar itself is dropping its `if`.

⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at Rust-compile time).

## STEP 4 — `:wat::core::keyword/of`

`wat/core.wat` — a stdlib macro whose entire purpose is building `Head<a,b>`. One caller:
`tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat`. Either it emits the form, or it retires
and its caller moves. Say which you chose and why.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | a type error's rendered type | `(:wat::core::Vector :- [:wat::core::i64])` — **copy-pasteable into source** |
| 2★★ | a monomorphic type renders | the bare name, never `Head<>` |
| 3★★★ | `(Head :- [])` ≡ `Head` at reference, ctor AND declaration | all three agree |
| 4★★ | a defservice expands, checks and DISPATCHES | a value comes back |
| 5★★ | a PARAMETRIC defservice (lru-svc, hologram-svc) round-trips | a value comes back |
| 6 | no `<` remains in any minted or rendered name | see below |

**Rows 1 and 3 decide it.** Row 1 is the whole point — a diagnostic you cannot copy back into your
program is teaching a language that does not exist. Row 3 is what makes the builder's rule true rather
than nearly true.

For row 6, do not grep — **impose the check**: temporarily apply the parked wall
(`git apply docs/arc/2026/04/109-kill-std/…` is NOT available; the patch lives outside the repo, so
instead add a temporary `debug_assert!` or eprintln at the two minting doors), run the floor, read what
still screams, then REMOVE the temporary probe. Report what it found.

## STOP triggers

- **STOP-1 — a renderer's output is consumed as an IDENTITY, not just displayed.** If some code
  compares or parses the rendered string, changing its shape changes behaviour. Report the site and
  what consumes it; do not change it blind.
- **STOP-2 — dropping an `(if (empty? proto-args) …)` branch changes a monomorphic service's
  behaviour.** That means step 2 is not actually done. Report which service and what changed.
- **STOP-3 — a golden's recapture is NOT the same call site** (different file, different column, or a
  genuinely different emitter). That is a real behaviour change hiding as churn. Report it.

## Boundaries

- The four renderers + their shared helper, `wat/service.wat`, `wat/core.wat`, and golden recaptures.
- **Do NOT apply the minting wall or `symbol-node`'s wall.** Next stone, once nothing mints.
- **Do NOT delete the angle PARSERS** (`split_type_params`, `canonical_callable_name`,
  `check.rs:5159`'s arm). They go once nothing mints AND nothing renders, with a green floor to prove it.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally. Use `./target/release/wat --check <file>`
  (~0.2s) and scoped `cargo nextest run --release -E '...'`.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Row 1 verbatim — the actual rendered type from a real type error — because that is the row that
proves the substrate stopped teaching a dead language. Then rows 2-6. The renderer helper's shape and
its callers. What you did with `keyword/of` and why. Every golden you recaptured, with the evidence it
is the same call site moved. Any STOP that fired, with the arm captured verbatim BEFORE you diagnosed
it. What surprised you.
