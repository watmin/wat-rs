# BRIEF — STONE 251.8d-ii (EIGHTH DRAW): the accessor join

**Drawn 2026-09-23 against `main` @ `3d8af02f3`.** Landable floor 6011/6011, clippy 0, census
`no STOP-8`, delta **3 / RECOVERY 0**, ⭐ **heresy ledger 220** (shape **B = 1**, A 94, E 125).
Converted floor **112**. Trajectory **3 747 → 416 → 283 → 161 → 112**.

## ⭐⭐ THE ADDRESS — ISOLATED WITH A PROBE THAT WAS PROVEN FIRST

⛔⛔ **The seventh draw's brief claimed a "measured" address that did not reproduce: its probe
matched `^\s+PASS` against LIVE `cargo nextest` output, which begins with an ANSI escape, so it
COULD NOT RETURN PASS and would have convicted whatever file was tested first.**
⭐ **`floor.sh`'s `clean.log` is already ANSI-stripped; a live pipe is not.** That is the whole bug.

**This time the matcher was proven before use** — on recorded logs it returns **6011 PASS / 0 FAIL**
on a green run and **5899 / 224** on a red one — **and every probe run carries a CONTROL test:**

| | build | target | control |
|---|---|---|---|
| baseline, unconverted | — | ⭐ **PASS** | PASS |
| ⭐ **`wat/spawn.wat` converted ALONE** | **0** | ⛔ **FAIL** | ⭐ **PASS** |

⭐ **All three verdicts demonstrated, build success checked, control green in the same run.**
`wat/` restored; the test passes again. **That is the address.**

## ⭐ The diagnostic — and the failing test is a NEGATIVE test

`probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time`, driving
`…_post_spawn_bogus_accessor.wat`, **expects exactly ONE unresolved reference** — the deliberate
`:wat::spawn::ProcessLaunch/bogus-field`. Under conversion it gets **TWO**:

```
:path ":wat::spawn::process/post-spawn"
:context "call head — not a builtin, not a registered function"
```

⭐⭐ **The LEGITIMATE accessor stops resolving.** ⛔ **So the visible symptom is a negative test
going red for the wrong reason — the bogus-field wall is FINE; a real name broke beside it.**

## The class — 33 of the 112, and it has not moved in two draws

| name | occurrences |
|---|---|
| `:wat::spawn::process/post-spawn` | **55** |
| `:wat::spawn::thread/init` | 6 |
| `:wat::spawn::thread/runner-count` | 5 |
| `:wat::spawn::process/runner-count` | 4 |
| ⭐ `:wat::core::Fault/of` | 4 |
| `:wat::spawn::process/max-message-bytes` · `thread/post-spawn` · the bogus control | 5 |

⭐ **Every one is a `Namespace::…/member` SURFACE JOIN** — `::` in the namespace, `/` at the leaf.
⭐⭐ **`Fault/of` is one of the THREE families 255.8 named under "what a door cannot express"**
(with `stdin-svc/start` and `Lru/new`). ⛔ **This is 255.8's wrong-join hole, and it has been open
since — listed as queued for five stones.**

## ⚠ WHAT IS **NOT** ESTABLISHED

1. ⛔ **Which side is wrong.** 255.8 recorded that `canonical_identity` writes `::` at the leaf while
   `reconstruct_call_path` writes `/` when the parent is a known type. **Which of the two should move
   here is UNMEASURED.**
2. ⛔ **Whether the cure is PERMISSIVE or RESTRICTIVE.** ⚠ Four consecutive stones have run
   permissive; **do not assume this one does** — a join that starts resolving could equally be a name
   that should never have resolved. **Determine the direction before designing the cure, and say
   which it is.**
3. ⛔ **Whether `wat/spawn.wat` is the only file in this class.** ⭐ **The single-file loop is ~2 min —
   run it on the other candidates rather than inferring.** (`Fault/of` is not in `spawn.wat`.)

## The work

1. ⭐ **Narrow inside `wat/spawn.wat`** by the same method. Report the discriminating form.
2. **Answer §1 and §2 above by measurement**, then cure through the identity door — one door,
   Keyword arm byte-identical (255.13's DUAL-ARM RULE).
3. ⭐ **Check `Fault/of` separately** — a different file, and 255.8 says a different registry family.
4. Non-vacuity, in one test:
   - the legitimate accessor resolves in **both** spellings, same identity;
   - ⛔ **`ProcessLaunch/bogus-field` is STILL UNRESOLVED in both spellings** — ⭐ **the negative test
     must go red for its OWN reason, and this stone must not cure it by accident**;
   - ⛔ a genuinely unknown member is still refused, with the same located reason.

## The gate

- ⭐ **Ledger drops and is RE-FROZEN in the same commit**; ⛔ **report the SHAPE MIX.** ⚠ **Shape B is
  ONE site** (`purity.rs::walk_rete_defn_callees`) and it **carries the calibration anchor** — the
  seventh draw deliberately left it uncured. ⛔ **If you cure it, re-anchor per the note in the test;
  never delete the assertion.**
- ⭐ **Converted floor 112 → ?** with ⭐ **FIXED/NEW against the seventh draw's 112.**
  ⚠ **Use `clean.log`, strip the WHOLE `FAIL […] (n/m)` prefix.** ⛔ **DO NOT PROMISE 112 − 33.**
- ⭐ **PROVE YOUR PROBE BEFORE YOU TRUST IT** — show it returning PASS **and** FAIL, and carry a
  control in each run. ⛔ **This is now a gate, because its absence cost the seventh draw an address.**
- `scripts/floor.sh` green on the **landable** state; clippy 0 (⚠ not cached); census `no STOP-8`;
  `scripts/replay/delta.sh` baseline **3**, ⛔ **RECOVERY non-zero is a STOP**.
- ⭐ **LAND THE CONVERSION ONLY IF THE CONVERTED FLOOR IS GREEN**, else `git checkout -- wat/`,
  rebuild, report. **Six draws have stopped correctly; all six were accepted.**

## ⛔ Operational traps

- ⛔⛔ **NEVER `git add -A`, NEVER invoke cargo, while a conversion is writing `wat/`.**
- ⛔ **Strip ANSI (`sed 's/\x1b\[[0-9;]*m//g'`) on ANY live tool output you match against.**
- ⛔ `git ls-files 'wat/**/*.wat'` → 31 of 64. Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- ⛔ The stdlib is `include_str!`'d — rebuild between steps and say you did.
- ⚠ A fast floor is a symptom (24 s = stdlib did not load; ~325 s healthy; ~460 s converted-but-red).

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⚠ One bookkeeping exception: `harvest_wrap_split`
  (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`). **Cite and report; never a pass, never
  re-run to clear. Name it either way.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Nineteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-five corrections across
  twenty-three stones**, four of them consecutive wrong addresses. ⭐⭐ **This brief's ADDRESS is
  measured with a proven probe and a control. Its MECHANISM and its DIRECTION are not.** Assume a
  twenty-sixth.
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

Shape **E** — ⛔ **125 sites, 78 in `check.rs`, UNTOUCHED across eight draws; the terminal cut needs
A+E at zero, not just heads** · `match_qq_head`'s **ledger blind spot** (literal at the call site —
⭐ **a real gap in the countdown that schedules the cut**) · `classify_expr`'s core-structural guard ·
`is_where_form` · `is_quasiquote_form` · `Ngram` · variant tags in declarations · 8d-iii.
