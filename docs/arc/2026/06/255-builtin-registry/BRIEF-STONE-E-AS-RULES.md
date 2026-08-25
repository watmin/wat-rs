# BRIEF — STONE E as RULES

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-E-AS-RULES.md` — read it whole, first.
It supersedes the codemod half of `DESIGN-STONE-E-the-string-home.md`; that older stone's door
table, its ruling on the rete mirror, and its ⊘ CORRECTIONS still apply and are cited below.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it.

**You may not spawn sub-agents.** Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not
commit, push, stash, revert, or checkout — leave your work uncommitted and report.

`cargo build --release` is yours (~19s incremental; longer for the wide `src/` change in step 4).
`cargo nextest`, `scripts/floor.sh` and clippy are NOT — the orchestrator takes those centrally.

```
systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G  -p MemorySwapMax=0 timeout 600 ./target/release/wat <args>
```

## ⛔ THE PRIOR ART IS NOT THE ONE THE OLD STONE NAMED

The old design pointed at `rename-kernel-to-spawn.wat` — a **char-walk** migration, and the reason
this stone had to be redrawn. **Your template is `wat-scripts/fixes/to-faithful-clojure-net.wat`**:
a working, shipped, rules-based codemod. 12 rules. Its header states the doctrine you are executing:

> *"The walk is PURE OBSERVATION (emits `:fix::Node` facts only); ALL classification lives in rete.
> rete stays pure — rules DEDUCE, the drive queries out + actions."*

Read it end to end before writing anything. **Its lines 273-282 are the exact tail your codemod
needs** — query, build edits, sort, apply.

**One thing you do DIFFERENTLY from it:** it mints its own fact base (`:fix::Node`, keyed on char
offset) because it predates `wat/grep.wat`. **You do not.** `wat/grep.wat` is stdlib now and gives
you `Node` / `Named` / `Span` / `Source` and `facts-of` for free. Minting a second observation layer
would be two fact bases for one job.

## The work

**Rewrite `wat-scripts/scratch-pad/BLOCKED-rename-core-string-to-string.wat` as a rules codemod and
move it back to `wat-scripts/fixes/rename-core-string-to-string.wat`.** The parked file's header
records why the char-walk version could not work; that header is superseded by the new one you write.

### The finder — two rules

```clojure
(:wat::rete::defrule :rn::core-string
  :when [(:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::String/starts-with? ?n ":wat::core::string::"))]
  :then [(:wat::grep::Match … :rule "core-string-to-string"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::core::String/concat ":wat::string::"
                                  (:wat::rete::core::string::subs ?n 20
                                    (:wat::rete::core::string::length ?n)
                                    :undefined "")))))])
```

and its sibling for `":wat::rete::core::string::"` → `":wat::rete::string::"` (prefix length 26).

**Verified by the orchestrator this session** — this rule, this shape, computes both the site and
the replacement:
```
old :wat::core::string::capitalize → new :wat::string::capitalize   span 17:19 → 17:49
```

⚠ **`subs` takes FIVE args.** `(s start end :undefined <default>)` — the mandatory undefined-point
that makes a partial op total on the rete surface (`vocabulary.rs:1333`). Omit it and you get
*"wants 5 args"* at `compile-all`, NOT at `--check`.

⚠ **The prefix lengths are 20 and 26.** Derive them, do not trust this sentence — an off-by-one
silently produces `:wat::string:tring::length` and every acceptance row below would still pass
except the diff.

### The applier

Per file: `facts-of` → insert the four fact vectors → `fire-rules` → `query q-match` → for each
`Match`, build `Tuple(offset, old-len, new-text)`:

```
offset   (:wat::fix::fix-text-offset-of {:line :col} lines)
old-len  (:wat::fix::fix-text-span-len start-span end-span lines)
new-text the "new" capture
```

★ **THEN SORT DESCENDING BY OFFSET.** `fix-text-apply`'s own doc comment (`wat/fix.wat:322`) says
*"apply a list of edits (in right-to-left order)"* — it splices in the order given, so an earlier
splice shifts every later offset. **Rete returns query results in NETWORK order, not source order.**
The char-walk never needed this because it walked in order; you do. `to-faithful-clojure-net.wat:275`
is the exact comparator to copy.

## The rooms — read in this order

1. **`wat-scripts/fixes/to-faithful-clojure-net.wat`** — whole. The template, and lines 273-282 are
   the tail you need.
2. **`wat/grep.wat`** — the stdlib fact base and `facts-of`. What you build on.
3. **`wat/fix.wat:316-345`** — `fix-text-apply` and the offset/len helpers. Untouched by this stone.
4. **`wat-scripts/scratch-pad/BLOCKED-rename-core-string-to-string.wat`** — the parked char-walk
   attempt and why it was a no-op. Read it so you do not re-derive the gap.
5. **`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-E-the-string-home.md`** — the SEVEN RUST
   DOORS table and the ⊘ CORRECTIONS. Counts are per-OCCURRENCE: `purity.rs` 5, `wat/string.wat` 22,
   `runtime.rs` 44, `check.rs` 31, `macros/eval.rs` 18, `expr_ir.rs` 10, `vocabulary.rs` 8.

## The sequence

1. Write the codemod (rules + applier). It must load and run against today's binary.
2. **Count before writing anything** — run the finder over the corpus and report the number.
3. **Dry-run to a `/tmp` copy and `diff`.** Byte-level. Not "it said renamed."
4. Apply to the corpus. Derive the path list from git, do not hand-write it.
5. Rename the seven Rust doors + `wat/string.wat`.
6. Build, iterating against the compiler — the diagnostics ARE the worklist
   (`docs/SUBSTRATE-AS-TEACHER.md`). A large fail count is the progress meter, not a crisis.

## The acceptance rows YOU run

- **Row 1 — the population, counted BEFORE applying.** The finder's Match count over the corpus.
  Compare to `:wat::core::string::` occurrences excluding comment-only lines. Report both; a
  discrepancy is a finding.
- **Row 2 — the dry-run diff.** On a `/tmp` copy, byte-level. Report its character: how many files,
  how many hunks, and confirm every hunk is a name and nothing else.
- **Row 3 — idempotent AS A QUERY.** After applying, re-run the finder: **0 matches**. This replaces
  "re-run and diff" and is the row the old mechanism could not have.
- **★ Row 4 — the TYPE is untouched.** `:wat::core::String` count IDENTICAL before and after.
  **Capture it before you start.** On the tree as you receive it the number is **4745** over
  `git ls-files '*.wat' '*.rs'` — verify that yourself rather than trusting it, then verify it again
  at the end.
- **Row 5 — per-door, per-OCCURRENCE.** All eight files, `grep -o … | wc -l` is 0. Report all eight.
- **Row 6 — the new name works, the old resolves to nothing.** `(:wat::string::length "x")` → 1;
  `(:wat::core::string::length "x")` → `UnknownFunction`.
- **Row 7 — the rete mirror moved, and its wall is real.** `(:wat::rete::string::length …)` compiles
  in a rule. Then break one `RETE_OPS` row deliberately, confirm `vocabulary.rs:1565` screams,
  restore it, and report the assertion text verbatim.
- **Row 8 — `(:wat::deporder::verify-stdlib)` → `[]`.**

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat-scripts/fixes/rename-core-string-to-string.wat` — created (moved from scratch-pad, rewritten)
- `wat-scripts/scratch-pad/BLOCKED-rename-core-string-to-string.wat` — removed
- the seven Rust doors + `wat/string.wat` — edited
- every `.wat` carrying the old prefix

No new verbs. `=` / `not=` for String belong to a sibling stone and DO NOT EXIST — if you find
yourself adding one, STOP.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **The dry-run diff touches `:wat::core::String` (capital S).** STOP — the finder is matching the
   type. Report the hunk verbatim.
2. **Row 1's count and the grep count disagree by more than the comment lines explain.** STOP and
   report both populations; that is a finding about the finder, not a rounding.
3. **The edits corrupt the file** — overlapping splices, a truncated name, a shifted offset. STOP.
   That is the sort contract and it is the one genuinely new hazard in this mechanism.
4. **A Rust door has occurrence counts the table does not predict**, or the build cannot be driven
   green. STOP and report the exact diagnostic; do not add a compatibility shim.

A STOP means: leave the tree as it is, write the report, end your turn. Never a licence to ship a
smaller version of a row.

## What you own that nobody can reconstruct

Row 1's two numbers and whether they agreed. The dry-run diff's character. The per-door counts.
Row 7's assertion text. The build's fail-count trajectory as you drove it down. And anything that
surprised you — a site the rules reached that the char-walk never could, or one they both miss.
