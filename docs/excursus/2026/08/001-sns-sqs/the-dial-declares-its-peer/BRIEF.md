# BRIEF — the dial declares its peer

**Read `DESIGN.md` beside this first.** It carries two corrections to what I first said (this is a **wat**
change, not `src/`; and the "silent crash" was my misreading), and the measurement that **refutes** the simpler
rule.

## The work, in one paragraph

`wat/service.wat` enforces a bijection between `:peers` and `:ephemeral` dialed peers, which forces every
service to dial in `:init`. A `connect` inside an `:impls` body participates in neither check, so a dial to an
**undeclared** surface is unrefused and kills the forked child at runtime. Add the third check: **a `connect` in
`:impls` whose argument resolves to an `Address<S::Op,S::Reply>`-typed field of this service requires
`S ∈ :peers`** — same `macro-error`, same wording family.

## The rooms — all in `wat/service.wat`

| where | why you are going there |
|---|---|
| `:889–905` | ⭑ **THE PROVEN SHAPE, check 1.** `_peers-missing` — a `foldl` over `peers-surfaces` calling `:wat::core::macro-error` with a concatenated message. Copy this structure exactly. |
| `:906–922` | ⭑ **check 2**, `_peers-extra` — the mirror, folding over `ephemeral-peer-surfaces`. Your check is the third in this family and should read like a sibling of these two. |
| just above `:889` | where `ephemeral-peer-surfaces` is derived from the `:ephemeral` field declarations (the `range 0 (/ ephemeral-len 3)` fold). ⭑ **This is the pattern for reading field declarations** — you need the same over `:durable` **and** `:ephemeral`, keeping the field NAME alongside the surface, because the connect-site walk matches on the accessor name. |
| `peers-surfaces` (same `let`) | the list your check tests membership in, via `:wat::vec::contains?`. |
| `wat-scripts/fanout/circuit.wat:370` | ⭑ **THE MUST-STILL-COMPILE CONTROL.** `:fanout::worker` declares `:peers [:queue::Queue :fanout::Seen]`, holds both in `:ephemeral`, and **redials `seen` inside its `-disrupt` handler**. Your check must accept this. |
| `wat-scripts/queue/sqs.wat` (queue `defservice`) | the must-still-compile control for the other direction: it holds `Address<queue::Queue>` — **its own** address — with `:peers [:wat::query::Store]` only. **Holding is not dialing**; this must not be refused. |

## Implementation sketch

```
;; a third sibling beside _peers-missing / _peers-extra
;; 1. addr-fields  : [(field-name, surface)] from :durable ++ :ephemeral where the
;;                   declared type is (Address :- [S::Op S::Reply])
;; 2. dialed       : walk the :impls forms; for each (:wat::kernel::connect X),
;;                   collect the surfaces of any addr-field whose ACCESSOR appears in X
;; 3. check        : every surface in `dialed` must be in peers-surfaces, else macro-error
_peers-undialed-decl (:wat::core::foldl
                       (:wat::core::fn [ok <- :wat::core::bool  ds <- :wat::core::String] -> :wat::core::bool
                         (:wat::core::if (:wat::vec::contains? peers-surfaces ds)
                           ok
                           (:wat::core::macro-error … ":impls dials Peer<" ds "::Op,…> but surface :" ds
                                                      " is not declared in :peers — add :peers [… :" ds " …]" …)))
                       true
                       impls-dialed-surfaces)
```

⚠ **The sketch is a convenience; the two existing checks are the contract.** Match how they derive surfaces
from declarations and how they word the error. **My sketches have been wrong seven times in this campaign** —
including one that would have caused the very trap-door I had ranked #1. Take the structure, verify every name
and every arity against `:889–922`.

## ⛔ Three ways to get this wrong

1. **Making it the strict rule** (`Address`-typed field ⇒ `:peers`, no connect walk). DESIGN's table shows it
   falsely refuses `sqs.wat`, `sns-fanout.wat` and `circuit.wat`. **Holding an address is not dialing it.**
2. **Walking `:init` instead of / as well as `:impls`.** `:init` dials are already covered by check `:913` via
   their `:ephemeral` peer; refusing them would reject the shape the entire corpus uses.
3. **Refusing a DECLARED handler dial.** ⭑ The strongest control is `sqs.wat`'s queue, which handler-dials
   the store **six times** (`:830 :866 :900 :1296 :1329 :1360`) with `:peers [:wat::query::Store]` — these are
   this session's own §2d redials. All six must be accepted. `circuit.wat:370`'s worker is the second control.
4. **Firing on a top-level `defn`.** `sqs.wat:1945`/`:2034` are helpers taking an `Address` **parameter** and
   `connect`ing it. They are not inside any `:impls` and the check must not see them — scope the walk to the
   `:impls` clause, never to the file.

## ⭑⭑ The proof — two probes, and they are a pair

| probe | expectation |
|---|---|
| ⭑ **refusal fires** — a `defservice` on `(:wat::spawn::process)` whose handler dials another service's `Address`-typed field, with **no** `:peers` | a **`macro-error` at compile time** naming `:peers` and the surface — **not** a runtime death |
| ⭑ **declared redial still compiles** — the same shape **plus** `:peers [:S]` and an `:ephemeral` `Peer<S>` | compiles and runs |

★ The first probe is the FINDING's variant A, which currently dies at runtime with
`Disconnected []` on the dialed peer and `RuntimeError ["unknown function: …"]` on the lineage. **After this
stone it must never run at all.**

## Verify

- `./scripts/floor.sh`, read the **Summary line** → green at **5239**. ⚠ **The new check runs over every
  `defservice` in the corpus at macroexpansion** — 305 `:satisfies` services across eight `.wat` homes. If any
  trips it, that is a **finding**, not a reason to weaken the rule: capture it and report.
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- `cargo nextest run --release --no-run`; `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- Happy path `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; chaos `… 0 0 7 0 0 0 0 0 500` → exit 0.
- `git diff --stat -- src/` must be **EMPTY**.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if the connect's argument cannot be resolved to a declared field syntactically** (e.g. it came
   from a message), STOP and report the shape. **Do not guess**, and do not widen to the strict rule to cover
   it — DESIGN rejects that with a measurement.
2. **STOP-2 — if any existing `defservice` in the corpus trips the new check**, STOP and name it with its file
   and surface. That is either a real latent bug (a finding worth more than this stone) or the rule is too
   broad; either way I want it before it ships.
3. **STOP-3 — do not touch `src/`.** This is a macro-time check; if you conclude it cannot be done in
   `wat/service.wat`, STOP and say why rather than reaching for the checker.
4. **STOP-4 — do not change `LociDiedError` or `Disconnected`.** The FINDING's second correction rules that
   out: it is honest as it stands.
5. **STOP-5 — `:init` is out of scope.** If you find yourself walking it, stop.

## Shape to copy

The two checks at `wat/service.wat:889–922` for structure and wording, and
`the-poison-tears/SCORE.md` for a stone that changed one rule and proved it with a before/after pair.
