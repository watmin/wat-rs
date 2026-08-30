# ⛔ NOTE — the `'` suffix does THREE jobs; `$native`/`$oracle` replaces exactly ONE of them

> **Builder, 2026-08-30:** *"we have been moving to using `name$native` instead of `name'` to
> denotate a native impl… `$oracle` is for wat defined."*
>
> Filed for arc 255. **Not scoped, not drawn.** Measured 2026-08-30 (post-compaction re-grounding)
> so a later stone starts from evidence rather than from the seam's summary — which was wrong about
> this on three counts, recorded below.

## The convention already EXISTS and is already APPLIED

It is not a proposal. It is written into the retired-name lint's own taxonomy
(`tests/lint/retired_name_justified.rs`, module header):

> *"Rete no longer uses `'` for the kernel: **public names are native, the wat reference is
> `$oracle`**."*

Applied population, measured:

```
5 live $native/$oracle pairs — the whole :wat::rete:: firing family
  fire-once · fire-rules · fire-rules-explain · insert · insert-all

src/runtime.rs:5617 — one arm serves the public name AND its native twin:
  ":wat::rete::fire-rules$native" | ":wat::rete::fire-rules" => { … }
```

And arc 278 stone 0z (`70fe856d`) already de-primed **24 IPC names across 302 files** via the
recorded codemod `wat-scripts/fixes/reclaim-ipc-prime-names.wat` (`send'`→`send`, `recv'`→`recv`,
`connect'`→`connect`, `Thread'`→`Thread`, …). **That migration is done.** Every `send'`/`recv'`/
`connect'`/`spawn-program'`/`run-thread'`/`mem-store'` still findable by grep sits in a `;;` comment
or in that codemod's own old→new rename table — **prose and history, not live code.**

## ★ THE FINDING — the prime is OVERLOADED, and the lint's runes say so out loud

Every live prime must carry a co-located `// rune:lint(retired-name) — <reason>`. **The runes are
the authority** (grep is not — see the corrections below). Reading all 18 runed sites in `src/`:

| job | the rune's own words | verbs | sites | `$native`? |
|---|---|---|---|---|
| **native impl** | *"live prime (arc 251 comparator-sort primitive); wat-level `sort`/`sort-by` wrap it"* | `:wat::core::sort'` | 5 | ✅ **YES — the only candidate** |
| **defmacro expansion target** | *"`readln'` is the `readln` defmacro's expansion target; same name, two forms (structurally required)"* | `:wat::kernel::readln'` | ~7 | ⛔ **NO** |
| **positional ctor idiom** | *"arc 294 9a: bare name is the kwargs macro, prime is the generated-only positional ctor"* | `:wat::program::Env'` · `EmptyEnv'` · `:wat::spawn::ThreadLaunch'` · `ProcessLaunch'` · `:wat::kernel::Frame'` | 6 | ⛔ **NO** |

**7 distinct live primed names. Three jobs. One is a native-impl marker.**

★ **The other two jobs are not "stragglers" — they are structural idioms where the prime marks a
GENERATED TWIN, not a native implementation.** Renaming `Env'` to `Env$native` would be a lie: it is
not native, it is the positional constructor the kwargs macro emits. Renaming `readln'` to
`readln$native` would be a lie for the same reason: it is the macro's expansion target, and the rune
says *structurally required*.

**So the migration the seam implied — "25 surviving primes move to `$native`" — does not exist.**
The real work is one verb, and a ruling on whether the other two jobs keep the prime or get their
own marker.

## The one real migration

```
:wat::core::sort'  ->  :wat::core::sort$native
```

5 sites, all runed, all in `src/`:

```
src/collection/transform.rs:282   const OP: &str = ":wat::core::sort'";
src/macros/eval.rs:505            | ":wat::core::sort'"
src/check.rs:20272                ":wat::core::sort'".into(),
src/runtime.rs:6023               ":wat::core::sort'" => {
src/rete/purity.rs:2046           ":wat::core::sort'",
```

★ It also **retires the rune that broke the floor** on 2026-08-30 — a co-located exemption dropped
when its arm relocated (`[[feedback_a_co_located_rune_is_attached_to_a_line]]`). A name that needs
no exemption cannot lose one.

⚠ **`sort'` is ALSO one of the unhomed verbs** (`WORKLIST-the-44-unhomed.md`) and one of the 59 in
the expand-time backlog. The rename and the homing land on the same verb — **draw them as one
stone, or the second one moves a name the first just moved.**

## ⛔ THREE CORRECTIONS TO THE SEAM — all three were mine, all three found by re-grounding

**1. "25 surviving primes" — CONTAMINATED.** The pattern `:wat::[a-z0-9:-]+'` matches English
possessives (`:wat::rete::compile-condition's`) and the *closing quote of prose* inside Rust
diagnostics (`"…canonical FQDN is ':wat::core::let'"` — that trailing `'` closes the quote; it is
not a prime). Of the 25, **7 are live.** The lint kills exactly these two false-positive classes
with a real predicate and says so in its header — I had the arbiter available and used grep anyway.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**2. "6 `-spec`" — the disk shows 4** (`foldl-spec`, `nth-spec`, `insert-all-spec`, and
`not-a-spec`, which is a fixture name meaning *not a spec*, so arguably 3).

**3. ⛔ "This DISSOLVES the W8 blocker" — IT DOES NOT.** The seam claimed the convention retires
`NOTE-the-firing-family-is-dual-implemented.md`. Re-reading that NOTE: **it was written knowing the
convention** — it quotes the `$native` arm verbatim as its central evidence. The blocker was never
the naming. It is three unanswered questions about **registry-vs-wat-`defn` shadowing**:

> 1. Does a registry entry shadow the wat `defn` for the FIRST-CLASS use, or only head position?
> 2. Should the public verb be homed at all, or only its `$native` half?
> 3. What happens to the shared arm when one of its two patterns is homed and the other is not?

A naming convention answers none of them. **The NOTE's own cheap probe still stands as the next
move there** — pass `fire-rules` as a *value*, then again with only `$native` homed; if the
first-class path is unaffected, question 2 answers itself and W8 reduces to an ordinary wave.

★ **The class:** I read a NOTE's *conclusion* ("dual-implemented, blocked"), matched it against a
convention I had just measured, and declared the blocker dissolved — without re-reading the NOTE to
see that it already cited that convention. A blocker's stated reason is the thing to re-read before
declaring it lifted. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`

## The open question this NOTE does NOT answer

**Do the other two jobs keep the prime?** Both are *generated-twin* markers, and both are
`rune`-justified today, so nothing is broken. But the prime now means three things, and two of them
are "this is the machine-made sibling of the name next to it." If that deserves its own suffix
(`$ctor`? `$expand`?), that is a **naming ruling, not a sweep** — and `wat-scripts/fixes/` is where
the migration would live once ruled, per R21.

⛔ **Do not draw a corpus rename on the other two jobs without that ruling.** They are load-bearing
idioms with working exemptions, not leftovers.

## What retires this NOTE

`sort'` → `sort$native` shipped (with its homing), and a ruling on the ctor/expansion-target prime.
Until then this file is the measurement, so the next drawer inherits it instead of re-deriving it
from a grep that over-counts by 3×.
