# SEAM — the ONE live breadcrumb. As of 2026-09-07. **GREEN, CLEAN, NO PEER IN FLIGHT.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE, DO NOT READ THE NUMBERS BELOW

```bash
git status --porcelain          # expect EMPTY
git log --oneline -1            # expect de2d9092a or later
grep -aE "^ +Summary" .floor/latest/raw.log
```

```
floor ....... 5233 passed, 0 failed, 18 skipped     clippy 0 errors     0 unpushed
peer ........ HALTED. Nothing in flight. `.pulsare/to-claude` holds a kind=halt.
```

⚠ **The floor takes ~175s and `every_wat_scripts_file_loads` finishes LAST.** Poll for
`^ +Summary` itself. Do NOT poll `pgrep -x cargo` — nextest runs as `cargo-nextest`, the loop
falls through instantly, and you will read a half-written log as a verdict. (Done today.)

## ★★★ WHERE THE CRUSADE IS — BOTH GATES ARE OPEN

The clojure-ification. **The match arm closed and 251.9 struck. Nothing blocks the head migration
except the two items under QUEUED.**

```
296 H/J/K/L   the enum wire · 26 enums into wat + a wall · aliases + an ORACLE · type-of    ✅
109 the arm   108 → 38 → 16 → 7 → 0 across five relands. 1869 files by codemod.            ✅
251.8b        Identifier STORES (ns, name)                                                  ✅
251.9         a symbol-headed declaration DECLARES. `head_fqdn` is the one door;             ✅
              the `_` arm FELL at every converted site (parse.rs 15 → 14)
109 kwargs    assertion-failed! TAKES KWARGS. 453 files by codemod. The positional form     ✅
              REFUSES. bare None 5818 → 650 (−5168); pairs 2588 → 17
```

★ **The enum migration SHRANK BY 70% as a side effect.** Bare Option/Result was 7366; the kwargs
stone DELETED ~5168 rather than rewriting them into a longer spelling and deleting them later.

★ 251.9's fixtures are the whole point and are worth re-reading before the migration:

```
(:wat::core::defenum :probe::Color …)   --check EXIT=0
(wat.core/defenum    :probe::Color …)   --check EXIT=1  →  now 0
```

ONE TOKEN apart, and the second is the target spelling. That stone was a scale model of the
migration it unblocks.

## ⛔ QUEUED — BOTH IN FRONT OF THE HEAD MIGRATION

```
head_of        src/runtime.rs:12232 — a Keyword-ONLY closure in `eval-with-defs!`:
  THE RESIDUAL     if head_of(f).map(|h| is_declaration_head(&h)).unwrap_or(false) { continue; }
                 A symbol head → None → NO continue → the declaration is registered by
                 `register_runtime_defs` AND THEN EVALUATED AGAIN.
                 ⚠ Unreachable ONLY because macro templates emit KEYWORD heads
                 (wat/core.wat:1348/1351/1393; 51 stdlib template sites emit a declaration
                 head). THE HEAD MIGRATION REWRITES THOSE TEMPLATES. The excuse has an
                 EXPIRY DATE and it is the very migration this unblocks.
                 NEEDS A PROBE — the current probe does not reach the eval path.

Option/Result  ⛔ NOT COMPLETED — **THE NEXT STONE**, and the builder's standing order is
  COMPLETION   "we made enums better, we should fully kill the old forms — finish that
                 migration before we start another."
                 THREE SPELLINGS ARE LEGAL AT ONCE:
                   :None  ·  :wat::core::None  ·  :wat::core::Option::None
                 runtime.rs:1700 / :8661 / match_arm.rs:162 accept them; 62 Rust sites
                 across 10 files keep the bare one alive.
                 RE-MEASURED post-kwargs   None 650 · Some 633 · Err 561 · Ok 365 = 2209
                                    qual   23
                 ★ BuiltinVariant (match_arm.rs:159) is a CLOSED enum of FOUR. Every other
                 enum in the corpus is already named through its enum — a bare
                 `:wat::service::Reply` returns ZERO. This is NOT a language hole; it is
                 four hardcoded exceptions that predate the enum work.
                 wat/core.wat:2125 declares them as VARIANTS, so the honest name is
                 `:wat::core::Option::None` → `#wat.core/Option.None {}`. The bare name is a
                 variant that SKIPS ITS ENUM.
                 ⚠ RULED: "i think :wat::core::None is illegal ... to communicate a none it
                 must be (wat.core/Option.None {})".
                 ⚠ OPEN FORK the builder has NOT ruled: does a unit variant become a
                 KEYWORD (`:wat::core::Option::None`, lands today, swept later with every
                 other head) or a CALL (`(wat.core/Option.None {})`, the end state, but
                 arrives coupled to the head migration)? THE CODEMOD'S TARGET TEXT DEPENDS
                 ON IT — ask before drawing.
                 ⚠ AND FINISHING MEANS THE RUST SIDE: rewriting the corpus while
                 `builtin_variant` still accepts ":wat::core::None" leaves the door open for
                 the next author. Pull the four special-cases and the `:None` arms so the
                 bare spelling is UNREPRESENTABLE, not merely unused.

assertion-span  ⚠ the kwargs refusal points at the MACRO, not the caller:
                 `:location wat/kernel/assertion.wat:34`, `:frames []`. It is an
                 `Option/expect`-past-end raise, so the expect's message IS the diagnostic.
                 The corpus is migrated, so the ONLY remaining consumers of that error are
                 humans/models typing the old form by hand — exactly who needs the span.
                 Partly fixable: `ast-span` is TOTAL, but `wat/grep.wat:67` records it
                 "returns keyword->i64 so it cannot carry :file".
#95            widen infer_list's gate. MEASURED: 10 lines, 0 compiler errors, 0 cascade.
5 dead-code    Coverage::Wildcard · pattern_coverage · ident_span · try_match_pattern_ast ·
warnings       substitute_many. NOT clippy (which is 0 and DENIED at the workspace root);
               these are rustc's. `ident_span` is in src/match_arm.rs, minted 2026-09-07, so
               "pre-existing" is true of the stone and UNMEASURED of the campaign.
               Corpse-or-door is a judgement PER SITE, not a reflex deletion.
```

## ⚠ RULINGS — do not re-litigate

- **The match arm is `[Variant {:k v} body]`, KEY-FIRST.** `{a :x}` is `let` order; `{:x x}` is
  PATTERN order. The flat positional clause is RETIRED.
- **A type wat uses is DECLARED IN WAT** — categorical, not "if it has bitten".
- **`TypeInfo` is ONE ROW, not N verbs.** · **Never key tooling on character case.**
- **The dot is modelled ONLY for variants**; a dot LEFT of the slash is namespace nesting.
- **`Type::member` / `Type/member` is a syntax we should never have had.** End state: per-namespace
  verbs. 109's domain.
- **A golden pinning a stdlib line: RECAPTURE IT, KEEP PINNING IT.** Do NOT extend
  `normalize_rust_source_span_lines` to `wat/*.wat` — that is the retracted option in stdlib
  clothes. (`109/NOTE-a-golden-that-pins-a-stdlib-line.md`)
- **⛔ SIDE BRANCHES DO NOT SERVE US** — 3 parked branches, 15 days, 0 merged.

## ⛔ THE FAILURE PATTERN — NOW TEN, ONE CLASS

**I COUNT SOMETHING CORRECTLY AND SAY THE WRONG THING ABOUT WHAT I COUNTED.** Every one caught by a
wall, a lint, the floor, the source, or the peer — never by me noticing:

```
25 enums / 26 · "no live producer" · "17 .wat.bad" (11) · "4 failures" (108) ·
"the codemod misses nested arms" · "[~@ {" as a population · "teach the arm reader"
  ⑧ THE BISECT — I read a first-bad commit as WHAT BROKE IT. 480f38d05 changed
    resolve/walk.rs `skip(4)`→`skip(2)`; the reference had been malformed since arc 278.
    ★ A BISECT NAMES WHERE A SYMPTOM BECAME VISIBLE. IT CANNOT ALONE NAME A CAUSE.
  ⑨ THE FRAMING — I reported "5816 :wat::core::None" as a neutral population count. It is
    the size of the ILLEGAL spelling, under a ruling the builder had already made. The
    number was right and the sentence was wrong.
  ⑩ THE PIPE, THIRD TIME IN ONE DAY — I read the kwargs stone's OWN load-bearing exit code
    through `| head -5` and reported EXIT=0. That was head's. The real answer was 101.
    ★ FM 20 is written about exactly this pipe and I hold a memory entry for it. KNOWING IT
    DOES NOT STOP IT. Run a gate unpiped and read `$?`, ALWAYS — mechanical, not remembered.
```

★ **THE CURE, PROVEN AGAIN TODAY: GO TO THE SOURCE, NOT THE BRIEF.** I was about to publish
"251.9 gates 7,816 declaration forms". `parse.rs`'s own doc stopped me: *"must be asked of a
POST-MACRO-EXPANSION form; `defn` expands to `def`."* Those 7,816 never reach that predicate. The
number was wrong in the direction that made my point bigger, and only opening the file caught it.

★ **AND THE ROOM MAP IS A CLAIM.** My 251.9 AMEND said "the `--check` failure travels the load
path" — right — then aimed at `preregister.rs`. The site was `src/types.rs classify_type_decl`,
outside `src/declare/*` entirely. The peer found it. **Naming that a second door EXISTS is worth
more than guessing where.**

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **GREEN AND QUIET IS THE MOST DANGEROUS STATE THIS FILE HAS EVER DESCRIBED** — there is no
> red to stop you and no peer to wait for, so nothing external will interrupt a wrong move.
> `git status` before anything else.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.`
