# FINDINGS — the chain-composition check

The replay converts each grok-rete `.wat` by running main's recorded migrations over it. Stone 0 made
every migration correct **alone**, against its fixture. This check proves the chain **in composition**:
chain(file@base) is compared with file@main over the 1,001 files main changed since `de827fb4c`, plus
the 488 files main never touched as a control.

The runner is `bootstrap/composition/run.sh`. It uses the FROZEN binary
`bootstrap/wat-replay-base-8a5b7eb20` and the exported tree `bootstrap/composition/tree-8a5b7eb20`, runs
at nice 19, and never touches the working tree. Runs land in `bootstrap/composition/runs/`. Before the
full run, it was timed on one file: 54 s, 46 s of it `positional-ctor-to-map`.

## Finding 1 — a later sweep co-updated an EARLIER tool (silent)

- `positional-ctor-to-map` (landing-order line 25) matches the qualified `:wat::core::Option::Some`,
  because `49f03f179` (step 26's own sweep) rewrote its match strings.
- A bare `(:wat::core::Some 1)` therefore prints UNRESOLVED and is left unchanged. Step 26
  (`bare-variant-to-qualified`) then leaves it **positional**, a form main retired.
- Each tool's fixture passes; only the composition fails.

## Finding 2 — an eval-based tool runs on TODAY's substrate (loud)

- Baseline run (landing order), step 23 `match-arm-to-bracket-map-pattern`, rc=2 on
  `tests/process/probe_arc278_init_crash_reason.wat`:
  `assertion-failed! takes kwargs … the positional form is retired` (`wat/kernel/assertion.wat:34`).
- Its `try-type-of` EVALUATES the file's declarations to learn field names. Today's runtime has retired
  a form that a LATER step (24, `assertion-failed-to-kwargs`) migrates.
- `order-v2` = landing order + two evidence-backed moves: assertion-failed before match-arm, and
  bare-variant before positional-ctor. With v2, match-arm completes (rc 0, 478 changed).

## Finding 3 — one unmigrated BODY poisons a whole file's type resolution

- v2 step 24: match-arm has **403 UNRESOLVED over 160 names in 77 files**. They are nearly all
  `<Svc>::<Method>Response::{Ok,RequestTooLarge,RequestMalformed}`.
- Main converted these arms (e.g. `tests/comms/probe_arc293_W2f_process_dials_thread.wat`), so they are
  CHAIN residuals.
- Mechanism: `try-type-of` = `eval-with-defs! (type-of E)` over ALL top-level `file-decls`.
  `decl-head?` admits pure type declarations AND `defservice` / `sift-rules-defsvc` / `defmacro`. A
  `defservice` whose `:impls` hold a base-era positional `::` constructor (`unknown callee:
  :probe::Echo::EchoResponse::Ok`) fails, so the whole eval fails and the fallback has no decls.
- That is a potential **cycle**: match-arm needs positional-ctor's and the dot flip's output, and
  positional-ctor also evaluates decls. Reordering cannot break a cycle.

### Two candidates, probed

- **(A) The era binary for the eval-based steps.** `git archive 480f38d05`, then `cargo build` in
  `bootstrap/era/` (needs the link `bootstrap/era/holon-rs` → `../holon-rs`; 52 s). The era match-arm
  on the era binary, on the example after steps 1–23: **UNRESOLVED 0, bracket arms 3 = main**. Today's
  gives 3 / 0.
  - Cost: positional-ctor's era text predates A′/(c), ~65 s/file.
  - Cost: era builds depend on today's `holon-rs` still compiling.
- **(B) Evaluate ONLY pure type declarations, at any depth; never bodies.** `file-decls` becomes a deep
  walk collecting `defenum`/`defrecord`/`defstruct`/`defsurface`/`newtype`/`typealias`/`typeunion`
  (`bootstrap/era/probe-B/match-arm-typedecls-only.wat`). Result on the example: **UNRESOLVED 0,
  bracket arms 3 = main**, on the frozen binary.
  - Indicator: 325 of 333 (name, file) pairs have an explicit in-file `defenum`, so (B) should reach
    ~98%.
  - **At scale (77 files, frozen binary, `probe-B/final.sh`): today 403 UNRESOLVED → (B) 16. (B)
    converted more arms in 73/77 files and fewer in none.** (Two earlier attempts measured nothing:
    one harness kill, and one relative-path bug in my runner that fed empty stdin.)
  - (B)'s 16, each classified:

    | n | where | what | class |
    |---|---|---|---|
    | 1 | `probe_diagnostic_c3…` | template arm in a `defmacro` | **not a residual**: main has 0 bracket arms there too |
    | 9 | `wat/service.wat` | `generated-template-without-fqdn` | **main hand content**: landing `480f38d05` hand-authored `init-arg-map-ast` in service.wat, and the era tool on `480f38d05^` cannot reproduce those 29 arms without it |
    | 1 | `probe_arc265_acronym_registry_svc` | `Waf::Op::CreateWebACL` | **(B) gap: acronyms.** Adding `:wat::string::declare-acronyms` to (B)'s heads → 0 UNRESOLVED, `{:req req}` = main (control: unmodified (B) on a copy reproduces the 1). `probe-D/`. ⚠ An earlier line here said "the era tool on `480f38d05^` reproduces the landing byte-for-byte". That was VACUOUS: main converted this arm at `498fbc1d6`, BEFORE `480f38d05`, so the era tool had nothing to change (`in` == `.orig`). The corpus conversion is split across `498fbc1d6` and `480f38d05`. |
    | 5 | `probe-m1-ann-erase2` | `PoolMsg`, `Echo::EchoResponse` | **(B) gap: program scope.** The file holds TWO programs: a parent, and a child inside `(:wat::core::forms …)`. Each declares its own `PoolMsg`/`Echo` ("SEPARATE typecheck universes", its header). The deep collector merges both, so the eval fails on the duplicate. Removing the child's copies → 0 UNRESOLVED. The real fix: a nested `forms` is its own scope. `probe-D/` |

## Finding 4 — positional-ctor's UNRESOLVED count is mostly noise (a CASE rule)

- `pascal-leaf?` asks `(= c (to-uppercase c))` of the leaf's first character. That is TRUE for `+`, `=`,
  `*`, `>`, so `(:wat::i64::+ …)` and `(:wat::core::= …)` are reported. It also reports collection
  ctors (`PersistentVector`) and rete patterns inside `quote`.
- v2 step 26 at 694 files: 1,896 reports over 306 names; the top ones are `PersistentVector` (270),
  `:wat::i64::+` (155) and `:wat::core::=` (128).
- It gates only the REPORT, never an edit, so the output is unaffected. But the channel cannot tell a
  real miss from noise, and it is a case rule, which the ruling forbids ("character case bears no
  meaning"). **For this step, the residual comparison against main is the instrument, not the count.**

## Finding 5 — the LANDING's match-arm leaked a type ACROSS FILES, and main carries it

- Main's `probe-m1-ann-erase2.wat` arms read `[:probe::PoolMsg.Setup {:deps addr}]` and
  `[:probe::PoolMsg.Work {:pair s}]`. Its own `PoolMsg` (both copies) declares `addr` and `s`.
  `deps`/`pair` are `probe-m1-phantom-d.wat`'s `PoolMsg :- [D I]`.
- **Reproduced:** the era tool (`480f38d05` tree and binary) on `480f38d05^` of erase2 ALONE writes
  `{:addr addr}` / `{:s s}`. With `phantom-d` in the same run it writes `{:deps addr}` /
  `{:pair s}`, byte-matching the landing. The landing changed both files. `probe-E/`
- So it is TOOL output, not a hand edit: one run's type state reached another file. The mechanism
  inside the tool is not traced yet (erase2 was listed FIRST, so it is not simple order).
- It is masked on main: the child program dies first at startup on an undeclared `:probe::CMsg`
  (main's binary, run: `UnknownNamedType … :probe::CMsg`). `CMsg` is declared only in the sibling
  `probe-m1-ann-erase.wat`. Grok-rete never touched erase2, so the file is broken on BOTH sides and
  its `EXPECT (green): "echo:z"` is false.
- **The class, not the case:** every arm main converted in the landing run could carry a same-named
  type from another file. The chain resolves in-file only, so it writes the declared fields. **The
  composition RESULT's residuals where chain = in-file declaration and main ≠ are this census.**

## Operational notes (all paid for today)

- Long runs are launched `setsid nohup`: the harness's memory guard kills tracked background tasks.
  Twice today it fired on a transient spike, and the only kernel OOM victim was an unrelated 29 GB
  `python3`.
- `pgrep -f '<script path>'` matches your OWN shell's text. Match the runner's argv prefix.
- A codemod piped EMPTY stdin dies with `"disconnected"`. That is never-ran, not "0 unresolved".
