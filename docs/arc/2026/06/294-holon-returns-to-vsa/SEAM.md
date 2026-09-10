# SEAM — the ONE live breadcrumb. 2026-09-10. **GREEN · CLEAN · PUSHED · NO PEER.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS.

```bash
git status --porcelain          # expect EMPTY
git log --oneline @{u}..HEAD    # expect EMPTY
grep -aE "^ +Summary" .floor/latest/raw.log
cargo clippy --release --all-targets -- -D warnings   # expect EXIT 0
```

```
floor 5318/5318, 22 skipped · clippy 0 · HEAD 1a022a278
```

## ⛔⛔ THE PEER IS GONE. DELEGATE LOCALLY, AND MEASURE YOURSELF.

`pulsare_yield` is not a channel. Local `Agent` riders only:

```
model: "sonnet"   EXPLICIT on every call (FM 12 — omit it and you spawn OPUS silently)
isolation         OMIT. Never "worktree" (FM 7-bis).
cwd               anchor /home/john/work/holon/wat-rs absolutely in the prompt.
no tool preamble  FM 16 — mentioning Bash/cargo availability triggers a hallucinated denial.
the brief MUST say: do NOT background a command and end your turn · do NOT contact any peer
```

★★★ **A MEASUREMENT IS YOURS. A DIFF IS THEIRS.** A rider's "STOP-5 did not fire" is worth nothing
if it answered a narrower question than you asked — one did exactly that yesterday, and the census
it skipped turned "two sites" into eight.

# ⛔⛔⛔ THE `:wat::*` BLANKET IS DEAD — `c3fefc5ab`

Arc 255's founding sentence, shipped after months.

```rust
-  if is_reserved_prefix(head) { return true; }
+  if crate::intrinsic::registry().contains(head) { return true; }
+  if head.starts_with(":rust::") { return true; }
```

A reserved-prefix name is a call head **iff the registry knows it**. No prefix is consulted for
`:wat::*`. The rung **falls through** on a miss — that single property is 97 files refusing instead
of 600. `:rust::*` defers to `UseDeclarations::covers`, which validates on the very next lines.

★ **THE WALL IS UNTOUCHED AND MUST STAY.** Userland may not *DEFINE* under `:wat::*`/`:rust::*` —
`resolve/registration.rs:129`, a DIFFERENT consumer of the same predicate. Seven rows in
`probe_arc255_the_reserved_prefix_wall_is_not_the_blanket` hold it, measured green both with the
blanket and with it deleted. **Deleting an acceptance is not deleting a refusal.**

## The road there — 8 stones, and what each closed

```
①′ membership facet   ②  normalize learns the type position    census 14 → 10
③  a declared TypeScheme is membership — 484 names, DERIVED, never a hand-list
④  the corpus's blanket-dependents → 0        ⑤-i  three phantoms retired
⑤-A self-peer is a special form (the FOURTH store: a literal inference arm)
⑤-E :wat::type:: is a TYPE-ONLY namespace     F+G  THE DELETION
```

★ It exposed real phantoms the blanket had hidden for months: `:wat::kernel::panic!`,
`:wat::string::=`, and `:wat::core::List?` — **called three times by `wat/core.wat` itself** with no
row, no scheme, no doc. Nothing had ever asked what it was.

## ⬜ THE LIVE WORK — the dot flip, and it is a PAIR

`:user::app::Box::Full` runs. `#user.app/Box.Full` renders. **The reader will not accept what the
runtime writes**, and both dot spellings are now cleanly REFUSED (they used to answer a silent
`None` — the blanket is why the seam ordered its death first).

```
✓ compose_variant / compose_variant_render     15 hand-rolled format!s → 1 door
✓ :wat::runtime::variant-parent-of             the substrate can be ASKED
✓ decompose_variant                            pairs the composer · cluster 1 of 8 routed
⬜ clusters 2–8                                 NOTE-the-variant-separator-census-eight-clusters
⬜ the flip + the codemod                       ONE landing, via fix.wat's STASH-DANCE
```

⛔ **THE FLIP IS NOT TWO LINES.** I asserted "exactly these two bodies" without counting. Read
`NOTE-the-variant-separator-census-eight-clusters-not-two.md`: **55 split lines, 20 flagged by
heuristic, 8 confirmed by READING.** Cluster 8 (`rete/expr_ir.rs:1321`) already uses the composition
door and is STILL broken by the flip, because its other half went to the general accessor.

⚠ **`identifier::path`/`leaf` must stay GENERAL** — they split every namespaced name; ~35 of the 55
are correct namespace splits.

★ The codemod ASKS (`variant-parent-of`) rather than matching. `:wat::cache::Cache::GetRequest` is a
`defrecord` of identical shape and answers `None`. **9,946 occurrences, 493 spellings** — a regex
renames the lookalike and nothing fails; it just means something else.

## ⚠ RULINGS — do not re-litigate

- **Turbofish has no form.** A comparison against it is not a weaker test; it is not a test.
- **An enum ctor is a MAP** · **`{:keys}` is one-shape; match is many-shape.**
- **A variant widens inside an ENUM's arguments** · **the join: same head → pairwise joins.**
- **`git commit <paths>`, NEVER `git add` then commit** (a new file needs `git add <that path>`).
- **⛔ NO SIDE BRANCHES** · **COMMIT LOCALLY OFTEN; PUSH ONLY GREEN.**

## ⛔ THE FAILURE PATTERNS — every one fired again yesterday

**① I NAMED A LIMIT THAT WAS A MISSING DOOR.** I wrote a codemod *"cannot"* tell a variant from a
record and drew a four-questions fork on it. The predicate existed and was load-bearing; it had no
wat surface. The builder: *"what query can't we make?… it's obvious when you ask."*
`[[feedback_i_named_a_limit_that_was_a_missing_door]]`

**② A COUNT IS NOT A ROSTER.** I added one test and silently disarmed a ratchet; the floor read
5305 → 5305 and only clippy saw it. Confirm gates ran **by name in the log**.

**③ `| head -3` SHIPPED AS A POPULATION.** Three abort fixtures in two artifacts; there were five.

**④ A VERB CAN PASS EVERY GATE AND BE UNUSABLE.** `variant-parent-of` was green, gated, exampled —
and refused a computed name, because I said "mirror `is-type?`". Found only by writing the caller.

★ **AND THE GATES THIS ARC BUILT CAUGHT ME REPEATEDLY** — `purity_mandated_examples` refuted three
axes; `registry_first_door_owns_every_handler_row` refuted a design; the loose-assert and
inlined-EDN lints caught four probes. **Read a red as a finding, not a chore.**

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **GREEN AND QUIET IS THE MOST DANGEROUS STATE THIS FILE DESCRIBES.** `git status` first.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
