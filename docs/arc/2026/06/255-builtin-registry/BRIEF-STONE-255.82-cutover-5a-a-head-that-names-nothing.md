# BRIEF — STONE 255.82: cutover 5a — a call head that names nothing is refused at check time

**Drawn 2026-10-02 against `main` @ `5555c55f2`.** **Executor: grok via pulsare, working solo** (it runs the floor). A
strike in the checker, its tests and a probe. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not
push**.

## The ruling (builder, 2026-10-02): H2

Cutover stone 5 (function and form heads become symbols, arc 251's 8d-iii) runs **gates first**: **5a** (this) closes
the unbound-head hole; **5b** censuses every Rust site that recognizes a head by keyword text and routes them through
one door; **5c** converts the stdlib, the corpus and embedded wat by the recorded conversion; **5d** makes keyword heads
illegal.

## The hole (measured by the orchestrator at `5555c55f2`)

`wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat` (committed with this brief) holds three
call heads that name nothing: a bare `(foozle 1)`, a dotted `(my.made.up.thing 1)`, and the no-slash wrong join
`(wat.core.Option.zzznope …)`. **`wat --check` exits 0.** Each fails only at run time as
`#wat.runtime/UnboundSymbol {:message "unbound symbol: …"}`. A keyword head that names nothing (`(:my::made::up 1)`)
**is** refused at check today, and so is a namespaced symbol `(wat.core/zzznope …)`. So the hole is exactly: **a symbol
head with no `/`**. The SEAM's 255.8 finding (the no-slash wrong join resolving) is one face of it.

## The work

1. **Find the dispatch:** where `infer_list` (`src/check.rs:2875`) handles a `WatAST::Symbol` head with no namespace,
   and what it does when the symbol is not a local (measure: it must be returning an unconstrained type without an
   error). Report the file:line.
2. **The legitimate bare heads (census first, report):** every bare or dotted symbol head in the corpus (`.wat` and
   embedded) that **does** resolve: a local bound to a function (`let`, `fn` params, match binders), a name a macro
   introduces, a special form or macro the reader/expander already knows by a bare name (if any). Heads inside `quote` /
   quasiquote are data, not calls. List each class with counts.
3. **The refusal:** a call head that is a symbol, not a local in scope and not a registered function, macro or special
   form, is refused at check time with `UnresolvedReference` (the error a keyword head gets today), naming the symbol and
   its span. For a dotted head whose `.`-split ends in a registered name (the wrong join, `wat.core.Option.expect`), the
   remedy names the slash spelling (`wat.core.Option/expect`). Route this through the same resolution the keyword head
   uses (one door), not a second lookup.
4. **Tests:** the probe's three heads refused at check with their names; a let-bound and a param-bound function called
   by a bare head still check and run; a quasiquoted bare head is still data; the probe becomes a `.wat.bad` (or moves
   beside a driven test) so the loader gate stays green.

## Gates

| what | how | expected |
|---|---|---|
| the probe | `wat --check` | refused, three `UnresolvedReference`, each naming its head |
| census | `scripts/replay/census.sh` pre and `--diff` after | every rc flip listed: each is a real unbound head the corpus carried (a finding, not a regression) |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running | all passed; the count against 6368 at `4160bdaf4` (`.floor/2026-10-02T09-46-26Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a corpus file that really calls an unbound head, the probe, its tests):
  capture it **verbatim**, cure the corpus file only if the head is a typo or a dead name (say which), run a **new**
  floor. Never re-run unchanged code for a green.
- **STOP-1:** more than 20 corpus sites are refused, or one class of bare head in item 2 is legitimate but the checker
  cannot see it as bound (a design question: how a name becomes bound). List them and STOP.
- **STOP-2:** the refusal fires inside quoted data or a macro template before expansion. Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.82-cutover-5a-a-head-that-names-nothing.md` beside this brief, commit it, **do not push**.
