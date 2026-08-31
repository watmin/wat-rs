# ⛔ NOTE — `Effectful + Deterministic` is TWO different claims, and 45 verbs hold it

> **Surfaced 2026-08-30 by the builder, from an argument rather than a measurement:**
> *"how is open file deterministic? if i create file, then open it, then delete it, then open it —
> the second call is a different outcome from the first? i get different results on the same input?"*
>
> Yes. Filed as a class; **not scoped, not drawn.** Two instances are corrected in stone total-T5
> because they broke a floor test; the other 43 are untouched and unexamined.

## How it surfaced

Stone total-T5 made `intrinsic_meta` derive purity and determinism from the registry. Two floor
tests promptly went red, both asserting `:wat::rete::deterministic?` is FALSE for
`:wat::io::IOReader/open-file`. That verb's registration declares:

```
@Purity        Effectful
@Determinism   Deterministic
```

with the reasoning: *"a real syscall with an observable OS-level effect (a new open fd); no external
actor's timing is awaited, so the outcome is **deterministic given an openable path** — the same
reasoning `resource.rs` gives `pipe`."*

★ **The tests were right and the registry was wrong.** They had been reading `false` for years via
default-deny — because the verb was absent from `intrinsic_meta` — and the first mechanism ever to
surface the registry's actual claim to that predicate found the claim was false.

## The defect in one sentence

**"Deterministic given an openable path" is a PRECONDITION, and a precondition does not rescue an
axis.** Every partial function is total on the subset where it is defined; that is not what these
axes measure. The identical move was refused for `:wat::i64::/` earlier the same day — it is
*"pure ∧ deterministic yet undefined at a zero divisor"*, and we ruled `@Totality Partial` rather than
*"total given a nonzero divisor."*

For `open-file` the varying thing is not even the domain — it is **the world**. Same path string,
different outcome, because the filesystem changed between calls. That is the same shape that makes
`uuid::v4` and `time::now` nondeterministic: **the output depends on state outside the arguments.**

## ★ THE DISCRIMINATOR, and it sorts the population cleanly

> **Does the RETURN VALUE depend on anything outside the arguments?**
> If yes → `Nondeterministic`, whatever the effect is.
> If no → `Deterministic` is honest, and `Effectful` describes the effect *elsewhere*.

Both halves are real, which is why the pair is not simply a contradiction:

```
RETURN VARIES WITH THE WORLD → nondeterministic
  IOReader/open-file · IOWriter/open-file    the file may or may not be there
  TempFile/new · TempDir/new                 a NEW path each call
  kernel::pipe                               a NEW fd each call
  listener · connect                         network state
  spawn-thread · spawn-process               a new handle each call
  HandlePool::new · HandlePool::pop          a new handle
  IOReader/from-fd · IOWriter/from-fd        depends on the fd's state

RETURN INVARIANT, EFFECT ELSEWHERE → Effectful ∧ Deterministic is HONEST
  println · pprintln · eprintln · epprintln  always nil; the effect is not the output
  raise! · assertion-failed!                 never return at all
  reset-sigusr1! · reset-sigusr2! · reset-sighup!   set a flag, return nil
```

`println` returning `nil` every time genuinely is deterministic. `pipe` returning a different fd
every time genuinely is not. **The pair is not the defect; the misfiling is.**

## The measured population

**45 registrations declare `@Purity Effectful` + `@Determinism Deterministic`.** By file:

```
src/intrinsic/holon/{engram,hologram,reckoner,subspace}.rs   14
src/intrinsic/kernel/{resource,stdio,ambient,abort}.rs       20
src/intrinsic/io/{reader,writer,fs}.rs                        7
src/rete/{export.rs,kernel/arm.rs}                            4
```

★ The 14 `:wat::holon::` entries are the interesting unknown — `Reckoner/observe`,
`OnlineSubspace/update`, `Hologram/put` are LEARNING operations that mutate accumulated state. Does
`observe` return the same thing given the same argument? Only if the accumulated state is part of
"the same input", which it is not. **They want reading, not assuming.**

## Why nothing here is drawn

The two `open-file` verbs are corrected in total-T5 because they broke a floor test — the minimum
that unblocks the stone. **The other 43 are not examined**, and sweeping them would mean 43
judgements made under the pressure of a stone that is about something else.

⚠ And note what the containment argument in T5's design did NOT cover: it proved
`ALSO_TOTAL = 0`, so the four-axis `where` fence admits nothing new. That is true and it holds. But
`:wat::rete::deterministic?` is a **standalone single-axis predicate**, and a single-axis reading of
a mis-declared verb is exactly what these two tests caught. **A containment argument must name which
consumers it covers.** `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## What a corrective stone would look like

1. Read all 45 against the discriminator above — return value, not effect.
2. Correct the misfiled ones at the registration site, each with its own reasoning.
3. ★ Expect the `holon` learning family to be the hard cases and to need a builder ruling: whether
   accumulated state counts as input is a semantics question before it is a determinism one — the
   same reservation `RULES`'s `:wat::holon::` disposition already records.

⛔ Not drawn. The builder rules whether, when, and in what order.

---

`DERIVAMVS NE MENTIAMVR.`
