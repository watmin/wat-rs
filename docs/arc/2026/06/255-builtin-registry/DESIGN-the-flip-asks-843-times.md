# DESIGN — ③b-ii: the flip asks 843 times

Drawn at `85dd4f452` (floor 5334/5334, clippy 0), after ③b-i put both variant registration paths
behind doors that compose rather than accept a name.

## ⛔ THE FINDING THAT REDREW THIS STONE: NEITHER INSTRUMENT IS COMPLETE

The codemod may not pattern-match a variant — `:wat::program::PeerKind::thread` is one and
`:wat::core::Record::def` is a surface method, and they are textually identical
(`::Capitalized::lowercase`). So it asks `:wat::runtime::variant-parent-of`. I proved that on four
names, and every one was right.

**All four lived in the stdlib.** Run against the whole corpus, one global ask confirmed **217**
names — and **345 of the 436 it declined are real `defenum` variants**, declared in corpus files
the derivation's runtime never loaded. It missed more than it found.

```
the ASK             sees macro-generated variants (a service's Op/Reply/Response enums are
                    never written as `defenum` anywhere) — but ONLY for loaded code
the DECLARATIONS    complete for source-written variants — but blind to the macro-generated ones
```

★ A four-row control that samples only the loaded half cannot see the unloaded half.
`[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]` — and this is the fifth time in
this campaign that a predicate has reported its own blind spot as a total.

## The complete instrument, proven before this design was written

`(:wat::load-file! <absolute-path>)` at TOP LEVEL (it is freeze-time — it has no expression
position) puts a file's own `defenum`s AND its macro expansions into the type env, and the
stdlib's answers survive alongside:

```
before the load   :counter::Request::Get     -- not a variant
after  the load   :counter::Request::Get     VARIANT of counter::Request
                  :counter::Response::Value  VARIANT of counter::Response
                  :wat::core::Option::Some   VARIANT of wat::core::Option    ← stdlib intact
                  :wat::core::Record::def    -- not a variant                ← method still refused
```

**One runtime per file sees both halves.** That is what neither instrument could do alone, and it
is why this stone asks 843 times instead of once.

## Phase ① — the derivation IS the census

For each corpus `.wat` file, generate a driver that loads it and asks about every namespaced
keyword token appearing in it; emit `:old::spelling :new.spelling` for each confirmed variant. The
union across all files is the complete rename set.

- The new spelling is **composed from the answer**, never edited out of the old string: the parent
  the substrate hands back IS the left half, so the dot lands where the substrate says the boundary
  is rather than where a `rfind` guesses. `:wat::cache::Cache::GetResponse::Ok` →
  `:wat::cache::Cache::GetResponse.Ok`, and the nesting is the substrate's, not a heuristic's.
- Candidate generation may **over-generate freely** — the ask is the filter. Every distinct
  namespaced keyword in a file can be offered. There is no predicate left to be wrong about, which
  is the entire point after five predicates that were.
- ⚠ Candidates must be **well-formed**: `keyword::from-string` refuses a leading colon (strip it)
  and refuses angle brackets (arc 109's wall). A malformed candidate aborts the run rather than
  answering `None`.
- The driver is generated because `load-file!` takes a LITERAL path at freeze time — it cannot be
  parameterised from stdin. Generated scaffolding is normally refused here; it is accepted only
  because the alternative is unioning two partial instruments, which is what produced the
  217-of-562 undercount.

The generator and the derivation script are recorded in `wat-scripts/scratch-pad/`, so the number
outlives the session that produced it. `[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

## Phase ② — the landing, and it does NOT need the stash-dance

`wat/fix.wat`'s header names the dance for a codemod that ships a NEW `:wat::fix::*` verb alongside
a Rust change. This codemod adds no verb — it uses `rename-keyword-exact`, which
`bare-variant-to-qualified.wat` (arc 296 N) already proved on the exact inverse migration. The
header says so itself: *"No Rust change in your strike? Skip the stash."* The order is:

```
1  run the codemod over the corpus with the CURRENT binary      corpus now spells `.`
2  flip identifier.rs — compose_variant AND decompose_variant   to `.`
3  flip the 28 `display` strings                                the ③a-iii manifest
4  cargo build --release                                        new checker + new corpus, consistent
5  scripts/floor.sh + clippy                                    one commit
```

⚠ Between 1 and 4 the tree cannot build — old checker, new corpus. That window is uncommitted and
must not be interrupted by a build. It is the same window the stash-dance exists to avoid, and it
is safe here only because nothing needs to compile inside it.

⚠ **Dry-run step 1 on a `/tmp` copy and `diff` it first.** The rewrite is a token rename; the diff
is the only artifact that can show it is exactly the intended structural change.

## ★ The 28 display strings must flip WITH the doors

`③a-iii`'s rune categories exist for this: `grep -rn 'rune:lint(one-variant-separator, display)'`
names every site that renders `Type::Variant` into prose a person reads. After the flip the
substrate must print `Type.Variant` or it prints a form its own reader refuses. They share no call
shape and nothing else in the tree could enumerate them.

## ⚠ Named risk: goldens that captured a display string

`.edn` goldens of VALUES are already dotted — `compose_variant_render` has always written `.`, which
is why `#wat.core/Option.Some` appears in today's error output. But any golden that captured a
**display string** carries `::` today and will carry `.` after. Those are not deferrals: the floor
names them, and re-capturing them is a row of this stone. A golden re-captured from the new binary
is honest; a golden hand-edited to match is not.

## Acceptance

```
phase ① emits a pair list; `Record::def` is ABSENT from it and `PeerKind::thread` is PRESENT
the corpus dry-run diff is read before the real run
every corpus file --checks after the landing        (843 files, the same census arc 255 F+G used)
floor via scripts/floor.sh, clippy -D warnings = 0, ONE commit
the one-variant-separator wall stays green          — the flip must not reintroduce a hand-rolled split
:wat::core::Option.Some registers                   — ③b-i's doors, now load-bearing
```

## What this stone does NOT do

Nothing is left for later. The corpus, the doors, the display strings and the goldens land
together, because there is no partial state that builds: a dot-spelled corpus needs a dot-reading
decomposer, and a dot-reading decomposer needs a dot-spelled corpus.
