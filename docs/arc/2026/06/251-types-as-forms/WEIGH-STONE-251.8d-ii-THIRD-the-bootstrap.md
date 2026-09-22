# WEIGH — STONE 251.8d-ii (THIRD): the STOP is ACCEPTED, and it bought four stones

Weighed against `14958ebb7` by independent re-run. **Executor: a fresh Opus rider, not grok**
(credits exhausted). ⛔ **Nothing landed.** Tree at `2dd489273` + the SCORE; `git diff -- src/` empty;
`git status --porcelain -- wat/` = 0 lines. **Recovery discipline: clean.**

## ⭐ THE STOP IS CORRECT, AND THE BRIEF'S CENTRAL GATE IS WHAT CAUGHT IT

The brief's whole point was *"it starts" is not "it works."* That is **exactly** where it died:
the converted stdlib **converts (64/64), builds (exit 0), loads, and `--check`s green** — and then
the converted codemod **converts nothing** (`rc 2`, 0 bytes), because `(:wat::kernel::readln)` dies
in stdin framing.

⭐⭐ **A conversion gate is not a loading gate, a loading gate is not a RUNNING gate.** Three stones,
three layers, each green one layer above the failure. **The brief predicted the layer and the rider
found it.**

## ⛔⛔ THE FINDING THAT MATTERS MOST — a codemod that makes the corpus GREENER as it BREAKS it

The rider reported, out of scope and unasked, that `src/load/loader.rs::match_load_form` matches
**Keyword heads only** (`Some(WatAST::Keyword(k, _)) => …, _ => return Ok(None)`), while the codemod
rewrites `(:wat::load-file! "x")` → `(wat/load-file! "x")`.

⭐ **I reproduced it, and it is worse than a no-op — it is a PASS:**

```
ORIGINAL  (keyword load head) : rc=1   "load: file not found: definitely-not-a-real-file.wat"
CONVERTED (symbol  load head) : rc=0
```

The converted load form is **never flagged at all** — not malformed, not unresolved. It is silently
not a load form. On a real corpus file the only surviving symptom is an `UnresolvedReference` for
whatever the loaded file would have provided; **if nothing from it is referenced, the file goes
GREEN while its load has silently vanished.**

⛔ **26 tracked `.wat` files use the six load forms. Zero in `wat/`** — so this stone is unaffected,
**and 8d-iii is not.** The flip will delete their loads, **and the corpus will get greener as it
does.** This must be closed BEFORE 8d-iii touches those 26 files.

⚠ **And the delta gate is BLIND to it.** 4 of the 179 sample files carry load forms; all four are
`CLEAN → CLEAN`. **Not a false "closed" — worse: a green that is green for the wrong reason.** No
count we have run for eight stones could have seen this.

⭐ **I nearly dismissed this finding on my own faulty instruments.** Three probes said `rc=1` for the
symbol form; all three failed for unrelated reasons I had written into them myself
(`MainSignatureError`, then a `MalformedForm` on my own `:wat::core::nil` body). **The rider was right
and my refutation was the broken thing** — the fourth probe, built from the codemod's real output
instead of my hand-written guess, reproduced `rc=1 → rc=0` exactly.
`[[feedback_ask_the_tool_that_owns_the_fact]]`.

## The mechanism of the STOP — verified

| claim | my result |
|---|---|
| `edn/render.rs` matches a raw string | ⭐ **confirmed**, `:2508`: `TypeExpr::Path(p) => match p.as_str() { ":wat::core::i64" => …` — a hardcoded table |
| `function/subsume.rs` likewise | ⭐ **confirmed**, `value_matches_type_by_name` compares the Path string |
| 255.8's `type_denotation` is not wired there | ⭐ **confirmed** — it exists (`render.rs:3614`) and is called from `types.rs`, `check.rs`, `collection/infer.rs`, **not from the table above it in its own file** |

⭐ **This is the recurring class the injected CLAUDE.md names outright** — *"a string comparison with
one side normalized and the other not"* — now on its fourth instance. The converted codemod writes
`wat.type/i64` (identity `:wat::type::i64`); the runtime table only knows `:wat::core::i64`.

## Gates

| gate | result |
|---|---|
| ⭐ converted codemod CONVERTS | ❌ **as drawn** — the stone's reason for existing |
| idempotence | ✅ 0/64, but **only under the rider's diagnostic probes** |
| floor | ❌ **RED 739/5959**, captured whole (`.floor/2026-09-22T03-36-00Z/ARM.txt`), **not re-run**. Second floor under probes **416**, also captured. ⭐ **Both verified present by me.** |
| census | ❌ 131 STOP-8, **zero in `wat/`** |
| delta | ✅ **18** reproduced pre-conversion |
| conversion scope | ✅ 64 paths, **byte-identical to the REDRAWN dry-run tree** — a third independent agreeing run |

## ⛔ NEW FROM THE WEIGH — the delta sample re-randomizes itself

The rider committed the list as invited (`delta-sample-179.txt`, 179 paths, sha `33ede76c…`). ⭐ I
diffed it against my own reconstruction from the 255.8 weigh: **179 each, and only 4 in common.**
The first three entries are identical, then they drift.

⛔ **"Every 12th of the census" is an INDEX over a GROWING list** (2145 files then, **2152** now), so
one inserted file shifts every later pick. **The eight-stone trend line `104 → … → 49 → 18` has been
measured over populations that drift apart as the corpus grows.**

⭐ **The finding survives anyway, and that is the reassuring part:** three near-disjoint 179-file
samples returned **18, 18, 16**. The conclusion is robust; **the instrument was not.** Committing the
list fixes it going forward — **this is the right cure and it should have happened six stones ago.**

## On the executor swap

A cold rider with none of grok's twelve-stone context **found a defect nobody had predicted, in code
the brief never mentioned, and stopped rather than forcing a green.** It disclosed two red floors
whole, answered the `Ngram` question by measuring **both directions** (transitional *and* symmetric —
⭐ the hazard does not vanish in 8d-iii, it **changes sides**), and corrected the brief twice.

⚠ Its one real miss was in my direction, not its own: it reported "symbol form rc 0" without the
file-level context that makes the rc depend on whether anything from the loaded file is referenced.
**That ambiguity cost me four probes.** A SCORE claim of the form *"X is rc 0"* needs the file that
produced it.

## VERDICT

**STOP ACCEPTED.** Pushing the weigh. The stone bought four stones and one correction:

1. ⛔ **`type_denotation` at the two runtime sites**, each with a probe that goes red without it.
2. **The 416-test remainder** (led by `wat::rete`, 195, losing operand types under the converted stdlib).
3. ⛔⛔ **The keyword-only `load-file!` head — BEFORE 8d-iii.** This is the one that corrupts the
   evidence rather than merely failing.
4. **255.8's wrong-join acceptance** — still open, still gating 8d-iii.
5. ✅ **The committed delta list is now the sample.** Never rebuild it by index again.
