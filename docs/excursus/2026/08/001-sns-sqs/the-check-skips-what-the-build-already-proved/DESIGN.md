# The check skips what the build already proved

**Tier B step C — the elision.** Excursus `001-sns-sqs`. Read first:

- `../tier-b-the-check-skips-what-it-already-proved/` — Tier B's FIRST attempt, **refuted**
- `../can-a-user-def-change-a-stdlib-verdict/FINDING.md` — the root
- `../a-mention-in-a-quoted-form-is-not-a-call/` — the repair
- `../every-sweep-names-what-it-reads/SCORE.md` — step A: which sweeps are closable
- `../no-stdlib-body-names-a-user-declarable-name/SCORE.md` — step B: the gate + the sweep control

⛔ **Tier B has been refuted twice.** This is the third attempt, and it is only drawable because A
narrowed the claim and B closed the surface and built the control. Do not widen it back.

## The sentence

> **Do not re-prove on every boot what the build already proved.**

## The prize, measured on a WARM cache (not inferred)

`WAT_BOOT_CENSUS=phases`, release, one-line user program, second run (cache warm), this host,
**single run — a reading, not a benchmark**:

```
3a-6b boot-cache-load      36.75   20.29%
8b    retired-syntax       11.90    6.57%
8c    restricted-call       3.23    1.78%
8d    def-position          1.80    0.99%
8f    body-infer          107.50   59.36%
      PIPELINE WALL       187.19
```

⭐ **`8b + 8d + 8f = 121.20 ms = 65% of the entire warm boot pipeline.** Tier A caches the stdlib
*expansion*; `check_program` is called unconditionally at `src/freeze.rs:1313` and runs on **every**
boot, hit or miss. (Step A measured the same three at 121.58 ms; run-to-run variance, same story.)

Target: ~187 ms → ~66 ms.

## What is settled, and must not be re-litigated

| | |
|---|---|
| **8b** and **8d's ALL-fns half** | closable by construction — AST + compile-time const tables (step A) |
| **8f** | closable on the spike induction (step A) |
| ⛔ **8c** | **NOT closable. It keeps running, at 3.23 ms.** Step A's verdict; B built its control. |
| ⛔ **8d's `forms` half** | `validate_def_positions_in_forms(forms, …)` is the USER program's own check. **Keeps running.** |
| the evaluated-position user surface in stdlib bodies | **zero**, held by B's gate |

## ⭐ Four things I established before drawing this — use them, verify them, do not re-derive blind

### 1. ⛔ THE PARTITION IS NOT THE BODY FILE

The obvious implementation — "skip bodies whose source file is under `wat/`" — is **wrong**, and
the witness shows why. `src/runtime.rs` is a body-file attribution carried by
**runtime-synthesized companions**, and for a user's own type those companions are the USER's:

```
src/runtime.rs  :probe::Counter'            schemes  :probe::Counter'
src/runtime.rs  :probe::is-Counter?         schemes  :probe::is-Counter?
src/runtime.rs  :probe::Bumper$core-record' typeenv  :probe::Bumper$core-record
```

A file-attributed partition **elides the user's own generated constructors and predicates.**
Partition by the bake-time function SET (or the `:wat::` path the namespace stone now gates),
never by `body.span().file`.

### 2. ⛔ `InferCtx.next` MUST BE REPLAYED, OR THE ELISION IS OBSERVABLE

8f is the only one of the three that takes cross-body mutable state:

```rust
check_function_body(path, func, scheme, &env, &mut fresh, &mut errors);   // check.rs:884
```

`InferCtx` (`check.rs:494`) is `next: u64` plus push/pop stacks. The stacks are per-function; **the
counter is monotonic and never reset.** Skip the stdlib bodies and every user body gets *different*
type-variable ids than it gets today.

**The fix makes the question moot rather than answering it:** record at bake time how many fresh
vars the stdlib sweep consumed, and advance `fresh` by exactly that on elision. Then the elision
is **observationally identical**, which is a far stronger claim than "var ids are probably
internal." ⚠ Verify the stacks really are balanced (empty between bodies) rather than assuming it.

### 3. The defclause door is closed by COMPOSITION, and I probed it

`preregister_defclause_in_env` (`check.rs:810`) writes **user** clause registrations into `env`
*before* the 8f sweep, and `defclause_regs` is one of the six live doors. So the door is real. It is
closed because a user cannot register a clause on a name a stdlib body can mention:

```
$ wat --check   # (defclause :wat::core::+ …)
#wat.runtime/ReservedPrefix {:message "cannot define :wat::core::+ — reserved prefix …"}
```

`register_defclause_from_form` routes through `parse_defclause_form(form, Privilege::User)`.
Composed with B's gate (stdlib bodies name only `:wat::` in evaluated position), a stdlib body's
clause lookup **cannot** reach a user registration. ⚠ Re-verify both halves; a composition argument
dies if either half moves.

### 4. The witness is ZERO — on ONE program, which is not enough

`WAT_SPIKE_WITNESS=1 WAT_SPIKE_WITNESS_ALL=1`, on
`wat-scripts/probes/arc-293/s3-probe-struct-satisfies-nature-struct.wat` (defstruct + defsurface +
extend-type + protocol dispatch), 31009 trace records:

> **`:wat::`-owned fn bodies probing a user-declarable name: 0, across 2177 bodies.**

⚠ **Read the instrument before quoting it.** The report's own `user-reachable` column is computed
from `WITNESS`, which is keyed `(door, name)` and **carries no body file** — it aggregates the
user's own body in, so it reads `user-reachable=132` on `schemes` and answers a *different*
question. Only `TRACE` (`(body-file, fn-path, door, name)`, `WAT_SPIKE_WITNESS_ALL`) can answer
this one. Two traps I hit and you will too: parametric method names are spelled
`(:wat::core::Seqable :- [:T])/seq` — a leading paren, still `:wat::`-owned; and `src/runtime.rs`
bodies are per item 1.

## The work

### 1. ⭐ WIDEN THE WITNESS FIRST — this is the stone's real content

One program proves nothing. Build an adversarial battery and run the TRACE query on each. At
minimum: a user `defclause` on a user name; `extend-type` of a **stdlib** protocol by a user type;
a user type flowing into a stdlib **generic**; a user `def` shadowing-adjacent name; a user
`defrecord`/`defstruct`/`defenum`; a user `set-redef!`. **The question each asks is the same:**
does any `:wat::`-owned body probe a user-declarable name?

⛔ **If any program yields a non-zero, C is REFUTED a third time and that is the delivery.** Report
the door, the stdlib fn, and the user name; land nothing. That outcome is worth more than the
121 ms, and it is the only outcome that is *already* known to be possible.

### 2. The elision, behind the verdict

Only if row 1 is uniformly zero. Skip 8b / 8d(ALL-fns) / 8f for the bake-time function set when a
verdict for **this build fingerprint** applies. Ride Tier A's existing machinery
(`src/freeze/boot_cache.rs` — fingerprint, encode-or-refuse, heal-on-refusal); do not mint a second
cache. ⚠ Derive the verdict from a world whose stdlib check is genuinely user-independent, and say
in the SCORE which world that was and why it is not circular.

### 3. The controls must survive it

B's pair (`the_restricted_call_sweep_still_walks_every_wat_body`,
`the_restricted_call_phase_records_a_hit_on_a_real_freeze`) must stay green — 8c is not elided.
⛔ **And the three elided sweeps now need what B gave 8c: a control that a USER body is still swept
by each of them.** Eliding the stdlib half must not quietly elide the user half. Prove it the way B
proved its own: break it on purpose and show the test reddens.

### 4. Measure the same way I did

Warm-cache `WAT_BOOT_CENSUS=phases`, before and after, same host, and say it is a single run unless
you run more.

## Scope wall

⛔ No `wat/` edits. Do not touch 8c. Do not touch 8d's `forms` half. Do not widen Tier B back to
"all four sweeps" — that claim is refuted twice and A explained why.

## The four questions

- **Obvious** — "don't re-prove at every boot what the build proved." ✅ The sentence is plain; its
  *preconditions* are not, which is why A and B came first and why row 1 leads.
- **Simple** — ⚠ **the weakest of the three.** It needs a verdict, a fingerprint, a partition and a
  counter replay. It rides Tier A's existing snapshot rather than adding a second mechanism, and
  the elision is one filter on three loops — but this is real machinery, not a one-liner. Say so.
- **Honest** — ✅ only if the elision is *observationally identical* (item 2) and the controls prove
  the user half still runs (row 3). Without both it is a silent hole, which is exactly what Tier B
  shipped-and-refuted twice.
- **Good UX** — ✅ user type errors still land; editing `wat/` changes the fingerprint; boot drops
  ~65%.
