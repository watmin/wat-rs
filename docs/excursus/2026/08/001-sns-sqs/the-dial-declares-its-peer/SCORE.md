# SCORE — the dial declares its peer

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `7d4a97843` (DRAWN). Did not commit.

```
     Summary [ 570.238s] 5241 tests run: 5241 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T06-55-12Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5239 → 5241** (+2 probes). Not a shrink.
Clippy: **CLIPPY=0**. `cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- src/` **EMPTY**. `LociDiedError` untouched.

---

## ⭑ THE HEADLINE — variant A is a compile-time refusal

```
probe::front: :impls dials Peer<probe::Mid::Op,…::Reply> but surface :probe::Mid is not declared
in :peers — add :peers [… :probe::Mid …] (the explicit s2s dependency DAG)
```

`#wat.macro/MalformedTemplate` at expand time. The FINDING's variant A **never runs**. Previously: `Disconnected []` on the dialed peer, `RuntimeError ["unknown function: …"]` on the lineage.

Same shape **plus** `:peers [:probe::Mid]` and an `:ephemeral` `Peer<Mid>`: compiles and returns `pong:hi`.

---

## WHAT LANDED

`wat/service.wat` — third sibling beside `_peers-missing` / `_peers-extra`. A `connect` inside **`:impls` only** whose argument names a durable/ephemeral field typed `(Address :- [S::Op S::Reply])` requires `S ∈ :peers`. Same `macro-error` family.

Walkers are **local fns** (symbol heads) so the F5 allow-list does not have to name them; they pass themselves as `self` because `let` is not `letfn`. Not `:wat::core::concat` (`:wat::vec::concat`). Not `:wat::core::<` (`:wat::i64::<`).

Not the strict Address-field rule. `:init` not walked. Top-level `defn`s not walked.

---

## Corpus — none tripped

`--check` of `sqs.wat`, `circuit.wat`, `sns-fanout.wat`: **exit 0**.
Floor green, including `every_wat_scripts_file_loads`. The new check ran at every `defservice` expansion in that corpus. **Zero trips.**

sqs queue handler-dials the store **six times** (`:830 :866 :900 :1296 :1329 :1360`) with `:peers [:wat::query::Store]` — all six accepted.
`:fanout::worker` redials `seen` in `-disrupt` with `:peers [:queue::Queue :fanout::Seen]` — accepted.

---

## Happy / chaos

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0
```

Chaos `2000 4 3 8192 true 1000 0 0 7 0 0 0 0 0 500` → **EXIT=0**, `bp-disrupt=500`, `disrupt-draws=254`, `disrupt-fires=17`, `distinct=8000;dup=0`. The worker's declared redial still works under injection.

---

## Residual (named, not covered)

A `connect` whose argument cannot be resolved to a declared Address **field** syntactically stays unchecked:

- address that arrived in a **message**
- `nth` of a `Vector` of Address (`sns-fanout` topic-worker `sub-addrs`)
- a **symbol** bound in a `let` (the init-local `store-addr`; top-level `defn` params at `sqs.wat:1945`/`:2034`)

This stone covers the field case — the corpus's shape and the one that bit. Not total coverage. Do not widen to the strict rule.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ refusal fires | `MalformedTemplate` naming `:peers` and `probe::Mid` |
| 2 | ⭑⭑ where it used to DIE | variant A **never runs** |
| 3 | ⭑ declared redial compiles | `pong:hi` |
| 4 | ⭑⭑ sqs six + worker seen | `--check` sqs + circuit **0**; chaos still redials |
| 5 | holding ≠ dialing | sns-fanout `--check` **0**; Topic/TopicWorker addresses are not impls-connect of Address fields |
| 6 | not the strict rule | walk is `:impls` `connect` sites; `addr-fields` only consulted there |
| 7 | `:init` untouched | walk is `ops` (`:impls`); init's `connect` not in that tree |
| 7b | top-level `defn`s untouched | `sqs.wat:1945`/`:2034` are not in any `:impls` |
| 8 | message is a sibling | same "dials Peer<S::Op,…::Reply> but surface :S is not declared in :peers — add :peers [… :S …]" family |
| 9 | no `src/` | **EMPTY** |
| 10 | `LociDiedError` untouched | **EMPTY** |
| 11 | floor | **5241** passed (+2 probes), no ARM.txt |
| 12 | clippy | CLIPPY=0 |
| 13 | happy / chaos | `distinct=8000;dup=0` · chaos EXIT=0 |
| 14 | corpus-wide | none tripped; `--check` of the three controls + floor |

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `7d4a97843` + the working tree.

```
floor   (my own run)  Summary [ 556.503s] 5241 tests run: 5241 passed, 22 skipped · 0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0
happy   distinct=8000;dup=0        chaos  distinct=8000;dup=0, exit 0
blast   wat/service.wat + 1 harness + 2 fixtures · git diff -- src/ EMPTY · LociDiedError untouched
```

**STRUCK. 14 of 14 rows on my instruments.** The runtime death is now a compile-time refusal.

## ⭑⭑ Row 1 — the refusal, my own run

```
TRUE exit=3   kind: MalformedTemplate
probe::front: :impls dials Peer<probe::Mid::Op,…::Reply> but surface :probe::Mid is not declared
in :peers — add :peers [… :probe::Mid …] (the explicit s2s dependency DAG)
```

★ **Row 8 is satisfied verbatim** — that wording is a true sibling of `:896`/`:913`, down to the trailing
*"(the explicit s2s dependency DAG)"*. The class now reads as three statements of one rule, not three rules.

And it landed as **floor tests**, not scratch probes — both pass in **0.78 s** together, so no wall exposure.

## ⭑⭑ Rows 4, 5, 14 — the must-be-accepted controls, `--check`ed by me

```
sqs.wat        --check exit 0     ← six handler dials of the store (:830 :866 :900 :1296 :1329 :1360)
circuit.wat    --check exit 0     ← :fanout::worker redials `seen` in -disrupt
sns-fanout.wat --check exit 0     ← holds Address<demo::Topic>, <demo::TopicWorker>, declares neither
```

⭑ **This is the row I expected a careless check to fail**, and it is the strongest evidence the rule is the
right one: the queue's six handler dials are *this session's own §2d redials*, and a strict
`Address`-field rule would have refused all three files.

Row 6 ✓ `addr-fields` is derived from the declarations but **only consulted at `:impls` connect sites**.
Row 7 ✓ by construction — the walk is over `ops`, and the source says so: *"⛔ `:init` is not walked (already
covered by check 2…)"*. Row 7b ✓ top-level `defn`s are not in `ops`.

## ⚠ THE COST IS REAL AND I MEASURED IT TWICE

| | floor | tests |
|---|---|---|
| before (four of my runs) | **523.1 · 523.6 · 527.5 · 529.3 s** | 5239 |
| grok, after | **570.2 s** | 5241 |
| **mine, after** | **556.5 s** | 5241 |

**+27 to +47 s — roughly +5 % to +9 %**, well outside the ~6 s band. And it is **uniform across the heaviest
tests**, not one regression: `wat::lint` +46.7 s, `wat::cli` +12.4 s, `wat::rete` +4.2 s / +5.6 s. That is
macroexpansion costing more everywhere, as expected for a walk at every `defservice`.

⚠ **Nothing tripped, but timeout margin shrank** — and this floor already lost a test to a 40 s wall one stone
ago (`.floor/2026-09-13T04-00-06Z/`). A 24 s test is now ~26 s. Worth knowing before the next slow test lands.

⭑ **And there is an obvious one-line optimization that is NOT present:** `impls-dialed-surfaces` is computed
unconditionally, so a service with **no `Address`-typed field at all** still pays the full `:impls` tree walk.
A guard — *"if `addr-fields` is empty, skip the walk"* — would avoid it. ⚠ **I did not quantify the saving and
will not guess it:** my only cheap instrument (*files mentioning an `Address` type*: 97 of 235 files containing
a `defservice`) **over-counts**, because helper signatures like `sqs.wat:1945` mention `Address` outside any
service. Measuring it properly means a floor run with and without the guard — a separate small stone.

## ★ A three-stone confirmation fell out of row 13

Chaos at **n=2000** with the **shipped** 500 bp:

```
disrupts=16 ; disrupt-fires=16 ; disrupt-draws=238 ; distinct=8000 ; dup=0
```

- **D2's equality holds** (`hits == fires`) at a second n and a second rate.
- **The shipped rate DOES fire at n=2000** — 238 draws, `P(zero) = 0.98²³⁸ ≈ 0.8 %`. ⭑ So the *"~77 % of runs
  fire nothing"* figure was specific to **n=50**, exactly as that stone's arithmetic said, and the floor
  sibling is the only place the low-draw regime applies.
- **The worker's declared handler redial survives live injection** with the new check in place.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ refusal fires | ✅ **my run** — `MalformedTemplate`, exit 3 |
| 2 | ⭑⭑ fires where it used to die | ✅ variant A cannot compile |
| 3 | ⭑ declared redial compiles | ✅ floor test, 0.78 s for both |
| 4 | ⭑⭑ sqs's six + worker's redial | ✅ **my own `--check`**, exit 0 |
| 5 | holding ≠ dialing | ✅ **my own `--check`** of sns-fanout, exit 0 |
| 6 | ⛔ not the strict rule | ✅ consulted only at connect sites |
| 7 | ⛔ `:init` untouched | ✅ by construction, stated in source |
| 7b | ⛔ top-level `defn`s untouched | ✅ not in `ops` |
| 8 | message is a sibling | ✅ verbatim family match |
| 9 | ⛔ no `src/` | ✅ EMPTY |
| 10 | ⛔ `LociDiedError` untouched | ✅ EMPTY |
| 11 | floor | ✅ my own run, **5241**, 0 FAIL |
| 12 | clippy | ✅ my own run, 0 |
| 13 | happy / chaos | ✅ `8000/0` both |
| 14 | corpus-wide, none tripped | ✅ three `--check`s mine + a green floor over 111 defservices |

## What I'd credit above all

**The residual is named with three concrete shapes, not hand-waved** — a message-borne address, an `nth` of a
`Vector` of `Address` (`sns-fanout`'s topic-worker `sub-addrs`), and a `let`-bound symbol (init's own
`store-addr`, and the `defn` params at `sqs.wat:1945`/`:2034`). ⭑ That third one also explains *why* row 7
holds for free: init's dial goes through a `let`-bound symbol, so the field-accessor test could not see it even
if `:init` were walked.

## What this hands the builder

The hole is closed and the mistake is **unwritable** for the field case. ⛔ Two things are named and not done:
the **guard** on the unconditional walk (unmeasured saving), and the **residual shapes** above, which stay
unchecked by design. And **D4 is still undrawn** — this whole line came out of its first probe.
