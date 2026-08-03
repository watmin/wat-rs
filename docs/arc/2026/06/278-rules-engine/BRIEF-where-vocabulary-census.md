# BRIEF — the `where` vocabulary census (scout, no build)

**This is reconnaissance. You change no source and build nothing.** Your deliverable is a list.

Spec context: `DESIGN-STONE-where-admits-only-rete-ops.md` § "THE MINT ROUNDS". That section carries a
provisional member list and says outright it must not be trusted — **you are what settles it.**

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the list is in your hands.

## The question

**Which operation heads actually appear inside a `where` expression in the corpus, and which of them
have no `:wat::rete::` mirror yet?**

Arming the fence refuses any head that is not `:wat::rete::<module>::<op>` (with the constructor,
accessor and `sym.functions` doors still open). So every un-mirrored head inside a `where` is a site
that breaks the moment we arm. That set is the mint list, and nobody has it.

## Why a grep cannot answer this — read before you reach for one

Three ways it fails, all of them already suffered on this arc:

1. **It cannot see nesting.** A `where` holds an arbitrary expression tree; the heads are at every
   depth. A line-oriented match cannot tell depth from adjacency.
2. **It cannot separate the subject from the harness.** These files are wat *programs* — `defn`,
   `foldl`, `range`, `sort`, `map`, `into` are driver code around the rules, not vocabulary inside a
   `where`. A whole-file count conflates them.
3. **Boundaries lie.** `\b` after a `?` cannot match (non-word followed by non-word), which produced
   two false zeros on this arc within the last day. And `grep -v '^\s*;;'` silently filters nothing
   when grep prefixes `file:line:`.

## The instrument — a form-tree walk

`wat/fix.wat`'s own machinery: `read-string` to parse a source file into forms, then walk the tree with
`with-children`. That is how the codemods traverse, it handles arbitrary nesting, and it never sees a
comment because comments do not survive the reader.

Walk each corpus file, find every `(:wat::rete::where <expr>)` form, and collect every **call head**
inside `<expr>` — at every depth, including heads inside nested `if` / `match` / `let` / `fn` bodies
and inside the arguments of other calls.

The corpus: `wat-scripts/perf/grid/where-*.wat` (nine families). **Enumerate the files yourself and
report the list** — do not trust that count. Also check whether any `where` forms live outside that
directory; if they do, say so and include them.

## Deliverable — three lists and a number

1. **EVERY distinct head found inside a `where`**, with its occurrence count.
2. **The MIRRORED set** — those already in `RETE_OPS` (`src/rete/vocabulary.rs`; read the table, do not
   assume its contents). These are pure codemod renames.
3. **The UN-MIRRORED set** — the mint list, and this is the deliverable that matters. **Bucket it by
   the three implementation shapes** the stone's "Three implementation classes" table defines:
   - **alias** — total already, rete name → same routine, zero new logic
   - **fallback surface** — partial, needs a second terminal handler taking `:undefined`
   - **form mirror** — a syntax form, not an operation
   State your reasoning per head in one line. Where you are unsure which bucket, say so rather than
   guessing — a wrong bucket sends a later rider at the wrong edit.
4. **The structural-form count** — how many `where` expressions use `match`, `fn`, or `cond`. The stone
   claims `cond` has *zero* corpus demand and a whole-file grep found 11 occurrences; **settle whether
   any sit inside a `where`.**

## Two specific answers I want called out

- **`=`** — a whole-file grep counted 127 occurrences of `:wat::core::=` in these files and it has no
  mirror. How many are actually inside a `where`?
- **`presence?` / `coincident?` / `cosine` / `dot`** — a grep found **zero** holon heads in this corpus.
  Confirm or refute.

## STOP triggers

1. **STOP-1 — `read-string` cannot parse a corpus file.** Report which and why; do not hand-parse it.
2. **STOP-2 — the walk cannot reach some construct** (a `where` built by quasiquote at runtime, say —
   `min-finding.wat` builds a `where` via `quasiquote`, so read it before you assume every `where` is
   literal in the source). Report what it cannot see. **An honest hole in the census is worth far more
   than a number that quietly omits it.**

## Blast radius

**Read-only, with one exception: you may create ONE scratch `.wat` file for the walk itself.** Put it in
`/home/watmin/.claude/jobs/3f09b1b2/tmp/`, **NOT** in `wat-scripts/`. That directory is walked by
`every_wat_scripts_file_loads`, which is a live gate for another rider working in this tree right now;
a new file there would break its run. The orchestrator places the durable probe later.

Run it with `./target/release/wat <your-file>`. Do not rebuild — another rider may be mid-link; if the
binary is momentarily missing, wait and retry rather than running `cargo`.

## Do not

Do not modify any source file. Do not run `cargo` at all — no build, no test, no clippy. Do not commit,
push, stash, or revert. Do not mint anything, do not touch `RETE_OPS`, do not arm any fence. **Your
entire output is a list and the reasoning behind it.**
