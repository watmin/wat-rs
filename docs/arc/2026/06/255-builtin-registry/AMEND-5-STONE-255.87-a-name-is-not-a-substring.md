# AMEND 5 — STONE 255.87: a name is not a substring, and a red that vanished is still a red

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `4e0436229`.
Commit locally on `main`; **do not push**.

Amendment 4 is accepted as measured: the default limits hold (`.floor/2026-10-03T10-31-41Z`, 6408/6408), the residual is
the six-run mean (1.107× shard 2, 1.090× keyed_gather), census has no flip. Two items keep 5c-ii open:

## 1. The delta's one NEW is a defect in the converted stdlib

`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` is rc 0 unconverted, rc 1 converted: `UnknownCallee
:robe/work::kwargs-check`. `wat/bracket.wat` reads the work function's `ast-name` and takes `subs` from index 1 to mint
`:<rest>::kwargs-check`: that strips the colon from a keyword and the **`p`** from the symbol `probe/work`. It is
`holon/CLAUDE.md`'s recurring class (a string operation on a name, one side assumed normalized). **Cure:** mint the
companion name from the work function's **identity** (`:wat::core::canonical-identity`, or the runtime's name-composition
door if one exists, e.g. `compose-variant`'s sibling for a companion; say which), never by slicing text. Then census
`wat/**` for every other `subs`/`string::concat`/`keyword::to-string` that builds or strips a **name** (not a message),
and route each the same way, each with a keyword/symbol pair test. `delta.sh` must then read NEW 0.

## 2. The doc-link lint's 60× swing

`no_broken_intra_doc_link_outside_the_frozen_ledger`: **TIMEOUT 30.004 s** on `.floor/2026-10-03T10-21-23Z`, **10.791 s**
isolated, **0.519 s** on `.floor/2026-10-03T10-31-41Z`. Same test, a 60× range, the reason unnamed. A red that does not
recur is not dispositioned by not recurring (no known flake). Find the mechanism: what the test does on a cold run that it
does not on a warm one (it may run `rustdoc` or `cargo doc` and depend on a cache in `target/`, or contend with another
test), from its source and from the kept logs. If its verdict or its cost depends on what ran before it, that is the
defect: make it independent of the cache (or of the order), so its cost is the same every run. **Do not raise its limit.**

Then the floor, clippy, `delta.sh` (NEW 0), `git status` clean. A STOP means STOP. Append to the SCORE, commit, **do not
push**.
