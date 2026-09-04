# BRIEF — STONE: the three special-form tables

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-the-three-special-form-tables.md` (sibling) first. It carries the measurements
this brief rests on, including the live `signature-of-defn` output proving most of the work is a
pure deletion.

## The work in one paragraph

Three tables answer "is this a special form": the registry (`#[wat_special_form]` →
`Kind::SpecialForm` + `entry.syntax`), `src/special_forms.rs`'s `build_registry()`, and a `const
SPECIAL_FORMS` declared inside `eval_apply`. The registry already wins at the reflection surface
for 23 of the 35 names, so those rows are dead text. Make both consumers ask the registry, move the
data for the nine names the registry cannot yet answer, and delete what is then unreachable.

## Rooms, in order

1. **`src/reflect/verbs.rs:83` `eval_signature_of_defn`, arms at `:201`, `:227`, `:247`.**
   Read all three before editing anything. Arm 201 (`!entry.syntax.is_empty()`) is why most of this
   stone is a deletion. Arm 247 contains `let _ = entry;` — the registry's answer discarded to
   re-ask the hand-list.
2. **`src/special_forms.rs`** — `build_registry()`'s 35 `insert(&mut m, …)` calls, `SpecialFormDef`,
   `lookup_special_form`, and the comment at `:172` recording that `match` no longer takes `-> :T`.
3. **`src/runtime.rs:5050`** — the local `const SPECIAL_FORMS` and its single use ~14 lines below,
   in `eval_apply`'s STOP-8 rejection.
4. **`src/reflect/lookup.rs:264`** — the other `lookup_special_form` consumer, and `:418`, which
   already tests `Kind::SpecialForm` and is the pattern to copy.

## The three buckets

The DESIGN names all three populations explicitly. Re-derive them yourself rather than trusting the
lists — here is the instrument, and **run it**:

```bash
perl -0777 -ne 'while(/\binsert\(\s*&mut\s+m\s*,\s*"([^"]+)"/gs){print "$1\n"}' \
  src/special_forms.rs | sort -u > /tmp/sf.txt
grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ --include=*.rs | sort -u > /tmp/reg.txt
comm -12 /tmp/sf.txt /tmp/reg.txt   # registered  (expect 32)
comm -23 /tmp/sf.txt /tmp/reg.txt   # not registered (expect 3)
```

⛔ A single-line regex **misses five** of these — six `insert(` calls span lines. The `-0777`
slurp above is why. This exact error cost the orchestrator a wrong count today.

Then, for each registered name, check whether its registration site carries an `@syntax` line. Those
that do are bucket 1 (delete the row). Those that do not are bucket 2 (move the sketch to an
`@syntax`, then delete the row).

## Bucket 2 — move the data, do not redesign it

For the nine names with no `@syntax`, add one at the registration site that **transcribes what the
sketch renders today, exactly**. Verify each with the substrate before and after:

```
(:wat::runtime::signature-of-defn :wat::core::if)  → must render identically after your change
```

⛔ If a sketch looks wrong to you, **write it down in your report and transcribe it anyway.**
Fixing a grammar is a different stone. A behaviour change hiding inside a deletion is exactly what
this stone must not ship.

## Room 3 — membership, and one honest exception

Replace the `const` with a registry query. `src/reflect/lookup.rs:418` and
`src/intrinsic/reflect.rs:384` already show the shape (`entry.kind` matched against
`Kind::SpecialForm`).

⚠ **`:wat::core::defn` is not a special form** — it is a stdlib macro that `apply` must also reject.
It has no registry row and cannot get one today. Keep it as an explicit named exception beside the
query, with a comment saying why it is not folded in and naming the FOURTH-registry fork. **Do not
silently drop it and do not pretend the registry answers for it.**

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if deleting a bucket-1 row changes any `signature-of-defn` output, STOP and report
  the name and both renderings. The DESIGN's claim is that arm 201 already shadows those rows; a
  change means that claim is wrong for that name.
- **STOP-2** — if any of the nine cannot be transcribed into an `@syntax` that renders identically,
  STOP and report which and why. Do not approximate.
- **STOP-3** — do not delete `src/special_forms.rs`, `SpecialFormDef`, or `lookup_special_form`.
  Three rows survive this stone; the file and its lookup survive with them.
- **STOP-4** — do not fold `:wat::core::defn` into the registry query, and do not fix any sketch's
  content.

## Verification — run the WHOLE binary, not a list of names I choose

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::reflection)'
cargo nextest run --release -E 'binary_id(wat::wat_lang)'
cargo nextest run --release -E 'binary_id(wat::function)'
cargo clippy --release --all-targets -- -D warnings
```

The orchestrator has three times this session handed a rider a list of test names that omitted
where the failures actually were. Run the binaries above whole.

## What to report

Your re-derived bucket lists (with the counts); the before/after `signature-of-defn` rendering for
each of the nine; whether arm 247 became unreachable and how you determined that; the Summary line
for each scoped run; and any sketch you transcribed that you believe is wrong.
