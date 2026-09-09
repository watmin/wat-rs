# FINDING — a codemod's selective tests run AFTER its joins, and nothing hoists them

Chased down after the builder asked: *"wut is wrong with rete?... what expression did you
create?... what questions are we asking?"* The answer to the first is **nothing**.

## What is actually true

`wat --grep` runs **one rete session per file** (`wat/grep.wat:383` — `overlay` re-seeds from
the compiled base, *"the isolation the DESIGN calls structural, not disciplined"*). The engine
hash-joins on shared bound variables and documents the degenerate case in its own type:
`JoinKey::{Empty, Unary, Nary}` with **`Empty = cartesian`** (`src/rete/kernel/session.rs:197`).

**The expression was the problem.** `topic-record-drop-nsubs.wat` ends each rule with trailing
`(:wat::rete::where …)` clauses. Those are **beta TestNodes over TOKENS** —
`src/rete/where_tree.rs` states its own contract as *"token bindings → candidate TestNodes"* —
so they filter **after** the joins are built. Meanwhile `durable-binder-nsubs` joins **three
`Node` patterns on one shared `?svc`**. For a form with C children that materialises ~C³ tuples
before `?hi = 0` or `?sn = ":demo::topic"` ever runs — and those two would have cut it to almost
nothing.

An **inline** constraint naming only its own condition's bindings is **alpha**
(`src/rete/matcher.rs:304`) and filters at fact entry. **wat has supported this all along:
1 of 93 recorded fixes uses it, 26 use trailing `where`.**

## Measured — A/B, same 5 rules, semantics proven identical

Equivalence first, on a pre-migration fixture (`329c9f512~1`), because a fast wrong rule is the
trap here: **both forms return the same 35 matches, identical `:line`/`:col` sets.**

| target | trailing `:where` | inline alpha |
|---|---|---|
| `wat/service.wat` @ 1g / 2g / 4g | **killed (137)** at every ceiling | **exit 0, 1–2 s** |
| whole corpus, 1809 paths, one shot @ 2g | **killed (137)** at 61 s — also at 8g, 12g, 16g | **exit 0, 41 s** |

★ It is **one file**: `wat/service.wat`, 3837 lines, the largest single form in the corpus. 1600
files pass in 2 g. Corpus size is irrelevant. And the trailing form does not "need N GB" — wall
time *rises* with the ceiling, so it **grows until killed**.

★ This makes every recorded fix's own instruction — *"list EVERY path"* — true again. It was
false for this rule at any ceiling, and chunking at 100 was masking that, not fixing it.

## ⛔ Four of my own claims died getting here. Each was killed by an instrument, not by thought

1. **"An unjoined `(:wat::grep::Source (?f <- :file))` is a cross-file cartesian"** — refuted by
   `grep.wat:383`. One session per file, so `Source` is one fact. I had read the fact schema and
   not the driver.
2. **Peak-memory numbers from `memory.peak`** — it is **not resettable on this kernel** and
   reported 8.1 G for one-second runs. Replaced with `--limit` bisection, where the smallest
   surviving ceiling *is* the demand.
3. **"Ten accumulated `failed` scope units caused the exit-3"** — no. It was a **parse error in
   my generated probe**. I had already written the false cause into a code comment; it is
   corrected there and the comment now says what was observed and what is *not* claimed.
4. ⛔⛔ **"PROVEN: inline completes where trailing dies"** — reported to the builder while the
   probe was compiling **ZERO rules.** The generator renamed `:tn::` → `:tni::` but left
   `(collect-rules :tn)`, so `:user::grep` collected nothing. **The fast version was fast because
   it did nothing.** The equivalence check is what caught it: 35 matches vs 0. The performance
   claim above is only worth anything *because* equivalence was measured first — and I ran it
   second, after already claiming the result.

## Should a user be allowed to make this mistake?

**No — and this one should not even be a choice.** For a constraint comparing one condition's
bound var against a literal, alpha and beta are **semantically identical** (measured above:
35 = 35). There is no workload where the trailing form is better. Leaving the placement to the
author is not flexibility; it is an un-hoisted optimisation wearing the costume of a style
preference.

The ladder, and how far the material goes:

- **Convention** — *"prefer inline."* Already failed: the knowledge sat in `matcher.rs:304` the
  whole time and 26 of 93 fixes still trail. A convention nobody writing codemods can see is not
  a rung.
- **A check at compile time** — `defrule` compilation already has the classifier it needs:
  `cond_has_deferred_constraint` (`matcher.rs:304`) distinguishes alpha-able from beta-only. So
  the compiler can *see* every hoistable `where` today and say so.
- **Hoist it automatically** ← the rung to aim for. Don't warn about a choice that has one right
  answer; remove the choice. ⚠ The safe subset is bounded and must be stated: `where_tree.rs`
  makes `exec_where` *"the sole authority on the verdict (and on raises for predicates we
  actually run)"*, so hoisting a predicate that can **raise** changes when and how often it
  raises. Restrict to **pure, total comparisons of one bound var against a literal** —
  exactly `classify_constraint_head`'s rows, and exactly what every codemod uses.
- **Remove the join** ← higher still, and specific to this vocabulary. All 21 fixes carrying the
  bare `Source` pattern hand-roll the same idiom: *"this node's parent form has head H."*
  `facts-of` could emit that as one derived fact per node (O(nodes), no join), and then the C³
  join cannot be written wrong because there is no join to write.

**Recommendation:** the automatic hoist, bounded to the pure-literal-comparison subset. It is a
constraint the engine can derive from what a `where` *is*, it needs no user to remember anything,
and it retires this failure for every codemod ever written — including the 26 already on disk.
