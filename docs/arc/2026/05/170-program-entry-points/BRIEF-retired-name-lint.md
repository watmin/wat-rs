# BRIEF — the retired-name lint: a wat name in a Rust string must be a name a user can type

## The work, in one paragraph

0z (`70fe856d`) dropped the `'` from 24 IPC names across 302 files and reclaimed the plain
names. It reached `.wat` keywords and Rust *symbol paths* — it did **not** reach names embedded
in Rust **message strings**. So the substrate still tells users about verbs that do not exist,
and the sharpest instance is the **checker itself**: `push_must_use_error`
(`src/check.rs:6930-6940`) emits a located `CheckError` saying *"a `send'` outcome must be
faced"* — naming a verb the user cannot type. R29 says the ruin educates; this one educates
toward a retired vocabulary.

Build the wall, then let it name the violators (R52 `QVOD LEX ACCENDIT`). A hand-sweep is a
stem-cut: 24t's cascade went **2530 → 20 → 3 → 0 by hand** precisely because nothing guards
Rust string literals (rename surfaces #4 and #5). A lint catches the stragglers this rename
leaves *and* the ones the next reclaim creates.

## Read in order

1. **`tests/lint/unused_span_justified.rs`** — the model. Copy its architecture wholesale:
   `collect_rs` walk of `src/`, a `LazyLock<Regex>` predicate, a `#[test]` that FAIL-lists every
   offender with `file:line`, a co-located `// rune:lint(<name>) — <reason>` exemption, and a
   `mod tests` proving the predicate **discriminates** (positive AND negative cases). It is
   auto-discovered into the `wat::lint` binary via `build.rs` — no registration needed.
2. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-no-inlined-edn.md`** — the detector
   doctrine, and it is load-bearing here: *"tighten the DETECTOR"*; pull each false-positive
   **class** out by the root as a predicate, **never** one-off runes.
3. **`src/check.rs:6926-6945`** — `push_must_use_error`, the sharpest violator.

## The predicate — two classes already proven, one still to kill

Start here (proven this session — English is rejected, primes are flagged):

```
a kebab/alpha identifier, then `'`, where `'` is NOT followed by an ASCII letter
```

| input | verdict | why |
|---|---|---|
| `don't` · `doesn't` · `Token's` · `the wall's` · `deftest's` | **ignored** | `'` followed by a letter — English possessive/contraction |
| `send': peer disconnected` · `accept' select` · `poll'/select'` | **FLAGGED** | `'` followed by punctuation/space |

**⚠ THE THIRD CLASS IS STILL OPEN AND IT IS THE BIG ONE — single-quoted prose.** The raw
predicate yields 112 hits and the dominant residue is a *closing* single quote, not a prime:

```rust
"wat: --check-output expects 'edn' or 'json'"        // edn'  ← closing quote
"canonical FQDN form is ':wat::core::nil' (arc 153)" // nil'  ← closing quote
"must read 'expects at least 2 arguments'"           // ...s' ← closing quote
```

Kill it with a predicate (a `'` that has a matching opener earlier in the literal is a quote,
not a suffix), **not** with runes. Expect more classes; each gets a predicate and a unit test.

## The three dispositions — every surviving hit is exactly one

- **FIX** — a retired name. The verb is `send`/`recv`/`connect`/`accept`/`poll`/`select`.
  Drop the `'`. This is the worklist.
- **RUNE** — a **live** prime. 24t's taxonomy: macro/verb pairs (`readln'`, `sort'`), the rete
  dual-impl (`fire-rules'`, `fire-once'`, `step-payload'` — unprimed is the wat ORACLE, primed
  the native kernel; **never collapse**), positional constructors, macro-minted disambiguators.
  The rune states which.
- **KEEP via predicate** — a historical reference in a *comment* to an arc by its name
  (`"connect' OUTCOME WALL"`). Comments are not user-facing; if the walk reads them, exclude
  comments in the predicate rather than runing 23 sites.

## The known-real FIX worklist (ground it; counts drift)

```
src/check.rs:6930-6940   recv' · poll'/select' · accept' · connect' · send'   ← the checker's remedy
src/runtime.rs:25496,25527,25576,25583   "send': peer disconnected"
src/runtime.rs:25660,25668               "try-send': peer disconnected"
src/runtime.rs:25943                     "recv' EDN decode failed: {}"
src/kernel/address.rs:140                "connect': rendezvous send failed …"
src/kernel/listener.rs:135,359,381,426   "accept' …"
src/kernel/spawn.rs:1070                 "peer_val must be RustOpaque(Thread')"
```

`check.rs:6938` (`"readln", "Datum/Eof/Stopped"`) is **already correct** — it was written after
the reclaim. It is the shape every other arm should match.

## Blast radius

`tests/lint/retired_name_justified.rs` (new) + the FIX sites above + runes on the live primes.
**No behaviour changes.** These are message strings and a remedy; no control flow moves.

## STOP triggers — reject and report; do not improvise

- **STOP-1.** A name you cannot classify as retired-vs-live from the disk. Do **not** guess and
  do **not** rune it to move on. `target/release/wat --check` a one-line fixture using the plain
  name (~0.2s): resolves ⇒ the prime is retired (FIX); `UnknownFunction` ⇒ the prime is live
  (RUNE). Report any that stay ambiguous.
- **STOP-2.** The predicate cannot separate a false-positive class without a rune. That means
  the class is not yet understood — report it with samples. Runing a class is the launder this
  brief exists to prevent.
- **STOP-3.** A "message string" that turns out to be a **symbol path** the code looks up
  (`sym.get(":wat::kernel::readln'")`). Changing it is a behaviour change, not a message fix —
  STOP, do not touch it.

## Gate

`cargo build --release --all-targets` (exit 0, zero warnings) and
`cargo nextest run --release -E 'binary_id(wat::lint)'`. **Run everything in the foreground.**
Do not run the full floor — the orchestrator weighs that centrally.

Then: `cargo nextest run --release -E 'binary_id(wat::lint)'` must be **GREEN** — a lint that
ships RED is not a wall. And prove it can go red: mutate one fixed site back to its prime
spelling and confirm the lint names it (`NISI FRANGAS, NIHIL PROBAS` — a lint that cannot go
red proves nothing).
