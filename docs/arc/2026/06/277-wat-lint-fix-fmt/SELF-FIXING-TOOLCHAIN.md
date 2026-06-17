# Building the tools that fix themselves

> **The doctrine of arc 277.** A tool is not done when it works. It is done when it can fix the very
> code that built it — and the proof is that code, in its final form, cleaned by the toolchain it
> embodies. The strange loop (arc 275: `deporder` written in the bad form it detects) made generative:
> the linter that finds bad forms fixes its *own* bad forms, and the git diff from gross to clean is the
> proof no argument can replace.

## The method

The campaign is `wat-lint` → `wat-fix` → `wat-fmt`, all wat, run on wat. But the *generative engine*
underneath it is a loop the work walks every time:

1. **The work reveals the gap (the reach-stumble).** You write real code in the substrate and it comes
   out gross. The grossness is not a failure of the author — it is the substrate naming a missing tool.
   The lint findings code did exactly this: `violation->finding` (in `wat/lint.wat`) is awful to read,
   and it is awful for two precise, nameable reasons.

2. **Each gap yields TWO artifacts, never one:**
   - **(a) the TOOL** — the thing that makes the right form expressible (e.g. `format`; labeled record
     construction).
   - **(b) the RULE** — a `wat-lint` rule that *detects the anti-pattern the tool replaces* and carries
     the fix to rewrite it. The tool makes the clean form *possible*; the rule makes the clean form
     *enforced* and *automatic*.

   This is the load-bearing move. A tool alone is a suggestion. A tool **plus** a rule that finds every
   place the old form survives and rewrites it is a *cure*. You do not hand-migrate; the linter finds
   them all (including the ones you wrote yesterday) and `wat-fix` applies the rewrite.

3. **Run the linter over the corpus — it fixes everything, including itself.** The very code that
   surfaced the gap (`violation->finding`) is now a test fixture: the linter detects its concat-abuse and
   its positional construction, and rewrites them with the tools that gap demanded. The tool eats its own
   tail, cleanly.

4. **The proof is the code in its final form.** Not a benchmark, not prose — the `violation->finding` in
   the committed history goes from gross (concat-chain + positional `Finding`) to clean (`format` +
   labeled construction), *by the toolchain it is part of.* The before/after diff is the proof the
   toolchain fixes itself.

## The three reach-stumbles this stone surfaced (the worked examples)

| The gross code (the reach-stumble) | The TOOL (makes clean possible) | The RULE (detect + auto-fix) |
|---|---|---|
| `(:wat::core::string::concat "load-order: " ref " (pos " (i64->string pos) ") …")` — literals + values interleaved, unreadable | **`format`** — `(:wat::core::format "load-order: {} (pos {}) …" ref pos …)` | **concat-abuse detector** — a `concat` chain mixing string literals with non-literal args → rewrite to a `format` call (literals become the template, args become the slots) |
| `(:wat::lint::Finding "load-order" ref 0 0 "error" msg "")` — positional, a reader cannot tell which arg is which (the `:None :None` opacity of arc 260, in the wild) | **labeled record construction** — kwargs-from-macros: `(:wat::lint::Finding :rule "load-order" :file ref :line 0 …)`, the companion macro expanding to the positional ctor (kwargs-is-always-a-macro; zero runtime cost). `Record/from {map}` the runtime sibling for dynamic fields | **positional-construction detector** — a record constructed positionally with ≥N fields (or any opaque scalar like `0`/`""`) → rewrite to the labeled form |
| `(if (= x "a") true (if (= x "b") true … false))` — a `HashSet` membership in disguise (the `deporder`/`fix.wat` bad form that started the arc) | **`HashSet/contains?`** (already exists) | **`nested-if-=-ladder` rule** (built, 277.1; auto-fix is 277.1b once `ast-end-span` exists) → rewrite to `(contains? (HashSet …) x)` |

Notice the column structure is the doctrine: **gap → tool → rule.** Every row is the same shape, and the
third column is what turns a one-off cleanup into a standing guarantee. The concat-abuse rule means no
future concat-chain survives; the positional-construction rule means no future opaque record survives;
the ladder rule means no future if-ladder survives. The corpus stays clean *by construction*, forever,
because the linter is always watching — and the first thing it catches is its own author's hand.

## Why the proof must be "the code in its final form"

The temptation is to *describe* the self-fixing property. Resist it (the `(/ c d)` of doc-writing:
asserting the result you smuggled in). The honest proof is operational and lives in the repo:

- `violation->finding` is committed **gross** today (concat + positional) — on purpose, as the fixture.
- The tools land (`format`, labeled construction) + their rules.
- `wat-fix` runs over the corpus and rewrites `violation->finding` — and `deporder`'s old ladders, and
  every other site — with no hand-editing.
- The diff is read: the findings code now *speaks* (a `format` template you read like a sentence, a
  `Finding` whose every field is labeled), and it was made to speak by the linter it is part of.

That diff is the thing. A toolchain that can point at its own source and say "this was gross, and I
fixed it, and here is the before and after" has proven the property in the only currency that counts.

## The recursion, named honestly

This is the arc-275 strange loop (`deporder` indicts itself) turned from *diagnosis* into *cure*:

- 275: the tool that *finds* bad structure was *written in* bad structure. It indicted itself.
- 277: the tool that finds bad structure, given the tools + rules each gap demands, **fixes** itself.

The loop closes, and closing it is the maturity line (#95/#96 Omerta / Again We Rise) at its full
extent: the language doesn't just self-host its execution (run on wat), or self-migrate its syntax
(`fix.wat`), or self-analyze its order (`deporder`) — it **self-corrects its own form**, and the
evidence is the form itself, final.

## Build order (tomorrow)

1. **`format`** (the tool) + the **concat-abuse rule** (detect + fix). Probe-first; `format` is a clean
   new `:wat::core::` primitive (or wat helper), no config.
2. **Labeled record construction** (kwargs-from-macros — the un-deferred slice of arc 260) + the
   **positional-construction rule**. The companion macro per `Record::def`; `Record/from {map}` if wanted.
3. **`ast-end-span`** (the primitive) → unblocks **277.1b** (the ladder rule's auto-fix).
4. **The sweep** — `wat-fix` over every `.wat`, including `violation->finding`. Read the diff. That is
   the proof; commit it as the proof.

> A tool that works is a tool. A tool that fixes the code that built it is a *substrate that takes care
> of itself* — and the proof was never the argument. It was always the diff.
