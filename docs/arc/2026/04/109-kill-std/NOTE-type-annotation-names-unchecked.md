# NOTE (arc 109) — a type name is validated in CALL position, never in ANNOTATION position

> ## ⊕ SIBLING FLAW, measured 2026-08-05 — the `:wat::` prefix exemption on FUNCTION heads
>
> **Builder: *"this is the fucking `:wat::` prefix check — in our backlog to attack, not a now
> thing."* Recorded here, not acted on.** It is adjacent to this note and deliberately NOT folded
> into it: this note is about **type** names and states outright *"it is NOT a `:wat::`-blanket
> effect."* The one below is about **function heads** and IS exactly that blanket effect. Same
> family, different axis; do not let the two get merged by a later reader.
>
> Measured with `target/release/wat --check`, one unknown head per file:
>
> | head | `--check` |
> |---|---|
> | `:user::nopeXX` | **REFUSED** |
> | `:user::NoSuchType::Variant` | **REFUSED** |
> | `:wat::kernel::printlnXX` | silent → runtime `UnknownFunction` |
> | `:wat::core::i64::plusXX` | silent → runtime `UnknownFunction` |
> | `:wat::rete::i64::plusXX` | silent → runtime `UnknownFunction` |
>
> **`--check` validates user-declared call heads and exempts every `:wat::*` one.** A typo in any
> builtin verb — kernel, core, rete — survives the checker and dies at runtime.
>
> This sharpens two existing records rather than contradicting them.
> `[[reference_check_is_not_a_complete_red_arbiter]]` and 278's own
> `NOTE-a-parametric-head-is-bare-a-path-is-not` both note that *"an unknown callee defers to a
> runtime `UnknownFunction`"* — **neither records that the deferral is scoped to the reserved
> prefix.** That boundary is the new fact, and it is what makes the gap sizeable: it is not a
> general checker limitation, it is a whole namespace class exempted.
>
> **★ THIS IS ARC 255's, AND 255 ALREADY NAMES THIS EXACT LINE OF CODE.** Builder's pointer, and
> the disk confirms it — `255-builtin-registry/DESIGN.md` states the cause verbatim:
>
> > *"with no registry to ask 'is this defined?', `resolve` falls back to
> > `if is_reserved_prefix(head) { return true }` — blanket-accepting any leaf, deferring wrong
> > names to a runtime 'unknown function'"*
>
> and the cure: *"The reserved-prefix blanket-accept hack is **deleted**; builtins resolve through
> the same path as user forms (registry/`sym` membership). One resolution path for everything."*
> 255's framing is the better one: this is not primarily a checker gap, it is the **asymmetry**
> that Rust builtins are the one callable kind that is unregistered, unqueryable and unreflectable
> — *"the undefined-func class dies as a SIDE EFFECT of fixing the real defect."*
>
> **Status: DESIGNED, PARKED.** 255's `CURRENT-STATE.md` breadcrumb is 2026-07-01 at HEAD
> `44c13e26`, floor `4285` — more than a month stale against today's `4348`. So the arc owns this,
> has the mechanism drawn, and is not in flight.
>
> **What today adds is only the EVIDENCE, and that is the reason to keep this block at all:** a
> measured boundary table, taken 2026-08-05, confirming the blanket-accept is still live and naming
> exactly which namespaces it covers. 255's design predates it. *(An earlier draft of this block
> called the general fix "unmeasured, possibly materially harder than the rete slice" and floated
> closing `:wat::rete::` alone off `RETE_OPS`. Both retracted: 255 has the general mechanism
> designed, and a rete-only patch would be a second resolution path in an arc whose whole thesis
> is ONE. The instinct to carve a cheap slice was reaching past a design already on the disk.)*
>
> Practical consequence today, worth carrying: **`--check` is not the arbiter for a
> reserved-namespace negative control.** A probe asserting "this bogus builtin head is refused"
> must be RUN. `#57` round 1b's scorecard row 3 says so for that reason.


**Filed 2026-07-26. A POINTER, not a decision.** Surfaced incidentally during arc 278's non-prime
IPC annihilation (the `Thread`/`Process` TYPE deletion) and parked here because it is **not** an IPC
concern and does **not** block that crusade. The ruling this needs already exists in 278 — see
*The 278 context* below. This note records the grounded flaw, the three exemptions any fix must
carry, and the unmeasured blast radius, so whoever picks it up does not re-derive them.

## The flaw, in one line

**A type name that does not exist is an error when it is a callee, and silence when it is an
annotation.**

## Grounded by a run (2026-07-26, `target/release/wat`)

```clojure
;; CALL position — validated, raises at check time
(:wat::runtime::return-type-of :s::NopeDoesNotExist')
;; => EXIT 3  #wat.check/UnknownCallee {:callee ":s::NopeDoesNotExist'" …}

;; ANNOTATION position — never validated, silent
(:wat::core::defn :user::take [n <- :s::NopeDoesNotExist] -> :wat::core::i64 7)
;; => EXIT 0, no diagnostic
```

**It is NOT a `:wat::`-blanket effect.** `:user::NoSuchType` is tolerated identically; there is no
prefix that grants a *reference* implicit trust. (The reserved-prefix machinery polices who may
**define** a `:wat::`/`:rust::` name — `TypeEnv::register_validated` → `resolve::gate`,
`src/types.rs:545`+ — a different axis entirely.)

**It is NOT a soundness hole.** An unknown name behaves as a distinct opaque *nominal* type: it will
not unify with anything else, so no value can be laundered through it. The moment the binding is
constrained, it surfaces:

```clojure
(:wat::core::defn :user::add-one [n <- :wat::core::i46] -> :wat::core::i64
  (:wat::core::i64::+ n 1))     ;; n IS used  => EXIT 1  #wat.check/TypeMismatch
(:wat::core::defn :user::take   [n <- :wat::core::i46] -> :wat::core::i64
  7)                            ;; n UNUSED   => EXIT 0  silent
```

So the exposure is exactly: **an unconstrained binding may name a type that does not exist, and
nobody says a word.**

## The mechanism (read first-hand this session)

`src/argspec/parse.rs:178` — the `<-` slot goes to `parse_keyword_type` → `parse_type_node`.
`src/types.rs:3681` — the parametric arm stores the head string verbatim:

```rust
if let Some(lt_index) = find_top_level_char(s, '<') {
    let raw_head = s[..lt_index].to_string();   // stored VERBATIM
    ...
    return Ok(TypeExpr::Parametric { head: raw_head, args });
}
```

Nothing on this path consults `TypeEnv`. The only error it can raise is `MalformedTypeExpr` — a
*syntax* complaint. And `src/types/error.rs` carries **18** error kinds, none of them an
unknown/unregistered-type variant: the check was never built, for any namespace.

## ★ Three exemptions any fix MUST carry (each verified passing today)

A naive "the name must be in the registry" predicate is **wrong on its face**:

```clojure
(:wat::core::defn :user::id<T> [x <- :T] -> :T x)                      ;; a TYPE PARAM, not a type
(:wat::core::defn :user::take [v <- :wat::core::Value] -> :i64 7)      ;; the universal top
(:wat::core::defn :user::take [p <- :user::Later] -> :i64 7)           ;; FORWARD reference
(:wat::core::defstruct :user::Later [a <- :wat::core::i64])
```

1. **Type parameters in scope** (rigid vars) are not registered types.
2. **`:wat::core::Value`** is *deliberately* unregistered — arc 278 R7 (`the universal top is a fixed
   point you point at, not a feature you build`). It is one branch, `src/types.rs:4252`
   (`if sup == ":wat::core::Value" { return true }`); registering it as a `TypeDef` would be **wrong**,
   because the top must be un-constructible. Check `:wat::Record` and any sibling top for the same shape.
3. **Registration order** — a forward reference within a file must stay legal.

The exemption set is the design question. It is not a keystroke.

## Blast radius — UNMEASURED

Turning this law on is `QVOD LEX ACCENDIT, REDIMIT` (278 R52): it lights every existing violator
across the corpus (1248+ `.wat`). Nobody has counted them, **including the author of this note** — do
not guess the number. The cheap sizing instrument is a one-off walk of every annotation reporting
which names resolve to nothing; run that **before** committing to the stone.

## The 278 context — the ruling already exists, one site short

`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-query-type-safe.md` already names this exact class
and already shipped the fix — for `return-type-of` only:

- **:28** — *"hiding **'this type does not exist'** behind a plausible string … **that is
  failure-masking**, the exact class this arc forbids (`RVINA ERVDIT` — the ruin must educate)."*
- **:50-55** — the runtime de-mask: raise on a name that is not a registered constructor/type,
  never echo it.
- **:56-58** — part 3, the check-time half: *"when `return-type-of`'s arg resolves to a (prime) type
  keyword, **validate it names a registered type**; if not → `CheckError`."*

**That stone landed** (verified by the run above — `UnknownCallee`, located, at check time). What is
missing is the same predicate at the `<-`/return annotation path. So this is a **278 stone extending
`query-type-safe`'s part 3 by one position** — not a new arc, and not owned by 109 beyond this pointer.

## Why it is parked here

It was found while deleting the non-prime `Thread`/`Process` TYPES, and it costs that crusade exactly
one thing, named so it is not forgotten: at **0z** (dropping the `'` from the primes) a surviving
non-prime type annotation on an *unconstrained* binding silently stops meaning nothing and starts
meaning the prime, with **no diagnostic**. One such site is known on the disk:
`tests/types/probe_arc214_lexer_primed_generic_head_control.wat` (its subject is the LEXER — it wants
any two-param generic head, and its `:wat::kernel::Thread<nil,nil>` is inert today).

This is a *silence*, not an unsoundness, and it does **not** impair the IPC cleanup. The IPC 0z
blocker remains the separate, already-recorded one: `CheckEnv::register` (`src/check/env.rs:266`) is
a bare `schemes.insert` with no duplicate gate — a real silent resurrection path for **verbs**.

## Kin

- `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-query-type-safe.md` — the ruling + the landed twin.
- 278 R29 `RVINA ERVDIT` — the system educates the caller; a silent accept teaches nothing.
- 278 R52 `QVOD LEX ACCENDIT, REDIMIT` — the corrected law lights every existing violator.
- 278 R7 — why `:wat::core::Value` is unregistered *by design* (exemption 2).
- `docs/arc/2026/06/278-rules-engine/DESIGN-reserved-prefix-one-gate.md` — ADJACENT ONLY: it polices
  who may **define** a reserved name, never whether a **reference** resolves.

---

## ADDENDUM 2026-07-28 — the silence was hit as an INSTRUMENT FAILURE, and the builder's ruling: **check must do more**

> **Builder, this session:** *"check doesn't do enough... we need it to do more."*

This note filed the gap as *"a silence, not an unsoundness."* Both halves still hold — and the
silence has now **cost something measurable**, which is the part worth recording.

### How it surfaced — `--check` was briefed as an existence arbiter and could not discriminate

Building the retired-name lint (`BRIEF-retired-name-lint.md`, arc 170), a rider had to answer one
question per site: **does this name still exist?** The brief handed it the obvious cheap arbiter —
`target/release/wat --check` a one-line fixture using the plain name, ~0.2s, resolves-vs-`UnknownFunction`.

**It does not discriminate.** Re-proven by my own run this session, and it is *wider* than the
non-parametric case this note already documents:

```clojure
[n <- :wat::kernel::TotallyBogus<wat::core::i64>]   ;; a bogus PARAMETRIC HEAD   => exit 0
[n <- :wat::kernel::RecvOutcome'<wat::core::i64>]   ;; a RETIRED PRIME spelling  => exit 0
[n <- :s::NopeDoesNotExist]                         ;; this note's own case      => exit 0
```

The retired prime is the one that stings: the *exact* question the tool was asking — *"is this
spelling still a name?"* — returns the same answer for a live name and a dead one.

### The escalation this justifies

A silence in the checker is not merely a missed diagnostic. It means **the substrate has no
queryable answer to "does this name exist?"** — so every tool that needs that answer must route
*around* the checker. That is the cost, and it is not hypothetical: it happened, in a brief I wrote,
and the rider had to find another way.

### What it used instead — recorded so nobody re-derives it, and its fragility named

The rider fell back to the **registration tables themselves** — grepping `env.register(…)` and the
`check.rs`/`runtime.rs` dispatch match arms — cross-checked against
**`wat-scripts/fixes/reclaim-ipc-prime-names.wat`**, the recorded 0z codemod, which names exactly the
24 retired names *and* documents the primes deliberately excluded from that rename.

That is strictly stronger than `--check` and it is ground truth today. **It is also the wrong shape
to depend on:** a recorded migration is a *historical artifact*, not a registry. It is correct until
the next rename, at which point it silently stops being the answer — the same rot this arc exists to
kill. Recording it as the current best instrument, not as the design.

### ★ THE DECOMPOSITION — two positions, and only one of them is the checker's job

The session separated a thing this note had held as one:

| position | example | can a checker see it? | the wall |
|---|---|---|---|
| **ANNOTATION** | `[n <- :s::Nope]`, `-> :s::Nope<T>` | **yes** — it is a parsed `TypeExpr` | **the stone this note is about** (extend `query-type-safe` part 3 by one position; carry the three exemptions above) |
| **STRING LITERAL** | `"a send' outcome must be faced"` | **never** — it is opaque text to every pass | **a lint** |

The second row is why the checker doing more would still not have covered today's work: a name in a
message string is invisible to the type system by construction. That wall **shipped today** —
`tests/lint/retired_name_justified.rs` — and it is the same shape as `unused_span_justified.rs`: for
emitted and embedded text, **the wall sits on the source, not on the checker** (the 24y meta-finding:
*wat's SURFACE is hardened; the RUST that implements and generates wat is not*).

So "check must do more" is the **annotation** stone, and it is now the only half still open. It does
not subsume the lint, and the lint does not discharge it.

### Blast radius — still UNMEASURED, and still do not guess

The sizing instrument this note already prescribes (a one-off walk of every annotation, reporting
which names resolve to nothing) has **not** been run. Run it before committing to the stone.
