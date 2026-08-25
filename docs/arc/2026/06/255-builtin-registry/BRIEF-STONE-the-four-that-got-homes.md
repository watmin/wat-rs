# BRIEF — STONE: the four that got homes they had not earned (phase 1, the ten verbs)

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-four-that-got-homes-they-had-not-earned.md`
— **read its `⛔ GROUNDED 2026-08-25` section first**; everything above that line is a superseded draw.

PRIOR ART TO COPY FOR SHAPE: `BRIEF-STONE-E-AS-RULES.md` + the recorded migration it produced,
`wat-scripts/fixes/rename-core-string-to-string.wat`. Same mechanism, four families instead of one.

---

## Your role

You are a **rider**. Your cwd is `/home/john/work/holon/wat-rs` — verify with `pwd` as your first
action and stay there; every path below is relative to it.

**Ending your turn ENDS you.** Nothing wakes you, and no notification is coming. Run every command
in the FOREGROUND and block on it: your turn ends when the numbers are in your hands, not when a
command is launched.

You make **text edits** and run the **cheap targeted checks named below**. The orchestrator builds,
floors, clippies, and commits. You may not commit, push, stash, revert, or `git checkout`.
**You may not spawn sub-agents.**

There is a `git stash@{0}` in this tree that must never be touched.

---

## The work, in one paragraph

Ten Rust-implemented verbs are registered under `:wat::core::` and none of them belongs there. Move
each to the namespace it earns. **The handler body does not change for any of the ten** — this is a
name migration, and that is what makes it one stone instead of four. Two disjoint mechanisms: the
`.wat` corpus moves by a **rules codemod you write**, and the `.rs` side moves by hand because in
Rust these names live inside string literals, which the codemod excludes by construction.

```
:wat::core::Uuid/v4               ->  :wat::uuid::v4
:wat::core::Uuid/v5               ->  :wat::uuid::v5
:wat::core::Uuid/from-string      ->  :wat::uuid::from-string
:wat::core::Uuid/to-string        ->  :wat::uuid::to-string
:wat::core::Uuid/nil              ->  :wat::uuid::nil
:wat::core::Uuid/version          ->  :wat::uuid::version
:wat::core::Uuid/rfc4122-variant? ->  :wat::uuid::rfc4122-variant?
:wat::core::regex::matches?       ->  :wat::regex::matches?
:wat::core::List/of               ->  :wat::core::List
:wat::core::char/of               ->  :wat::core::char
```

The last two are **finishing** a migration, not starting one: every other collection type is already
its own constructor (`(:wat::core::PersistentVector 1 2 3)` evaluates today; `(:wat::core::List 1 2 3)`
is `UnknownFunction`). `:wat::core::List` and `:wat::core::char` already exist as TYPES and already
pass `--check` in annotation position — both verified on a fresh binary. Registering them as
constructors is exactly what `PersistentVector` has always done: one name, type in annotation
position, constructor in head position.

**Do not rebuild.** The `target/release/wat` you have is correct for the codemod — the stdlib is
compiled into the binary, so on-disk `.wat` edits do not affect it. The orchestrator rebuilds once,
centrally, after you report.

---

## ACT 1 — the `.rs` rename (130 live occurrences across 22 files; 4 gravestones stay)

### The registrations — change the string in the attribute, nothing else

| room | what |
|---|---|
| `src/intrinsic/uuid.rs` — 30 lines, listed below | seven `#[wat_intrinsic(":wat::core::Uuid/…")]` + their `const OP:` + doc `///` + `@example` |
| `src/intrinsic/regex.rs:14,29,30,38` | one attribute + `const OP` + doc + `@example` |
| `src/intrinsic/list.rs:1,4,18,31,32` | one attribute + doc + `@example` |
| `src/intrinsic/char.rs:13,28,29,36` | one attribute + `const OP` + doc + `@example` |

`src/intrinsic/uuid.rs` lines: 40 53 54 55 64 78 79 80 88 116 129 130 131 138 159 168 169 170 177
193 201 202 211 222 223 230 246 258 259 266.

### ★ THE FOUR DOORS THAT DO NOT CALL THE VERB — THEY EMIT IT

These are the rooms a call-site census cannot see, and three of them construct the call rather than
make it. **Read each one before you edit it.**

| room | why you are there |
|---|---|
| `crates/wat-reader/src/parser.rs:397,402,406` | the **`\c` char literal desugars to `(:wat::core::char/of "x")` at PARSE TIME.** No corpus file contains that call; every `\c` in the corpus becomes one. Miss this and `\c` emits a call to a name that no longer exists — **and not one `.wat` file changes.** |
| `src/runtime.rs:21367,21369,21373` | `to-wat` renders a char back as `(:wat::core::char/of "c")` — the output side of the same round-trip. It must agree with the parser or `\c` stops round-tripping. |
| `src/closure_extract.rs:1994,2001,2005,2011,2015` | portable-value encoding **emits** `Uuid/from-string`, `char/of` and `List/of` calls into a wire format. |
| `src/rete/purity.rs:2213` | a frozen **alphabetical NAME list** — data, not a call. `:wat::core::char` sorts into the same slot `:wat::core::char/of` held. |

### ★ THE RETIREMENT TABLE — the substrate's memory of its own rename

`src/remedy/retirement.rs` maps a retired form to its replacement so the error at an old name
**names the new one**. Its header states the rule: *"Each HARD CUT stone appends its retirement
entry at the arc's ship time."* Ten names are being hard-cut, so **append exactly ten entries**,
same shape as the 25 already there:

```rust
// Arc 255 — the four families leave :wat::core:: for the namespaces they earn.
RetirementEntry { retired: ":wat::core::Uuid/v4", replacement: ":wat::uuid::v4", note: None },
…
RetirementEntry { retired: ":wat::core::char/of", replacement: ":wat::core::char", note: None },
```

and add the matching rows to the module header's `## Arc history` table.

Measured, so you know what "done" looks like: `(:wat::core::Char "x")` — retired by stone 242.1 —
today yields a **`MalformedForm`** reading *"':wat::core::Char' is retired (Stone 242.1); use
':wat::core::char' instead"* with a `:remedies [… :kind :retirement]`. That is the target shape.
A bare `UnknownFunction` on an old name means the table was not fed.

⚠ Do **not** add entries for `:wat::core::string::*`. Stone E moved those and left the table unfed;
that is a separate finding on E's ledger and is out of this stone's scope.

### The type-checker and the purity classifier

| room | why |
|---|---|
| `src/check.rs:17581,17603,17615,17626,17636,17646,17656,17666,17678,17682,17686` | `register_builtins` — one `env.register(":wat::core::…")` `TypeScheme` per verb. This is where the new name acquires a type. |
| `src/check.rs:3015,3016,3018` and `15272,15274,15294` | `infer_list`'s special-cased `":wat::core::List/of"` arm and `infer_list_of`'s own `callee:` string. |
| `src/rete/purity.rs:10,250,489,490,491,492,551,563,1601,1602,1608,1613` | classification arms + a unit test. Note `:wat::core::List?` and `:wat::core::List/length` at line 551 are **NOT in scope** — only `/of` moves. |
| `src/rete/purity.rs:262` | reads `head.starts_with(":wat::core::regex::")` — a PREFIX door. It already carries `:wat::string::` beside it from stone E; give it `:wat::regex::` the same way. |
| `src/value/value.rs:311,339` | doc comments naming the constructors. |
| `src/string/mod.rs:35`, `crates/wat-doc/src/lib.rs:1062` | prose / a doc-parser test fixture string. |

### The tests

`tests/value/wat_arc220_char.rs` (11 12 13 14 58 80 106 163) ·
`tests/program/probe_arc213_program_edn_roundtrip.rs` (16 26 110 144 163 167) ·
`tests/collection/list.rs` (4 32 39 54) · `tests/types/uuid.rs` (68 80 135) ·
`tests/rete/probe_fence_names_the_head.rs` (15 **87**) ·
`tests/rete/probe_arc278_seq1b_list_hofs.rs` (56 135) · `tests/rete/probe_arc278_6a_purity.rs` (5) ·
`tests/program/wat_arc278_sigma_fn_purity_gate.rs` (15) · `tests/kernel/wat_string_ops.rs` (95) ·
`tests/collection/probe_seq_container_parity.rs` (70).

⚠ `tests/rete/probe_fence_names_the_head.rs:87` asserts an **exact error-message string** containing
`':wat::core::Uuid/v4'`. That assertion moves with the name.

### The four files the `*.wat` and `*.rs` globs both miss

```
tests/program/wat_arc278_sigma_fn_purity_gate_presence_nondeterministic.wat.bad : 4
tests/cli/wat_mcp__persist_def_value.jsonl                                      : one Uuid/v4
docs/USER-GUIDE.md   : 2558 2561 2566 2570 2575 2587 2588 2602 2603 2605 2617 2638 2646 2648 3674
README.md            : 257
```

`README.md:257` also names the deleted `string_ops` — fix that half of the line while you are there.

### KEEP — do not touch

- The **four retirement-record comments** naming `:wat::core::Char/of` —
  `crates/wat-reader/src/parser.rs:402`, `src/check.rs:17682`, `src/closure_extract.rs:2001`,
  `src/runtime.rs:21369`. They record that stone 242.1 killed that name; they are history, not live
  code. (The stone's own draw said there were three and called them a live duplicate; both halves
  were wrong.)
- Everything under `docs/arc/**` other than this arc's own DESIGN/BRIEF/EXPECTATIONS. Past
  INSCRIPTIONs, SCOREs and BRIEFs are immutable record.

The one `Char/of` that DOES move is prose: `wat-tests/holon/char-round-trip.wat:3` says
`` `(:wat::core::Char/of "x")` `` and should say `` `(:wat::core::char "x")` ``.

---

## ACT 2 — the codemod (239 keyword leaves, `.wat` corpus)

Write **`wat-scripts/fixes/rename-four-families-to-their-homes.wat`**.

Copy `wat-scripts/fixes/rename-core-string-to-string.wat` wholesale and change its rules. It already
carries every piece you need: the `Node`/`Named`/`Span`/`Source` join, the **keyword kind guard**,
the `:user::grep` finder entry point, the `:rn::q-match` field-destructured query, `edits-of`,
`convert-one` with its **descending-offset sort**, and the `readln` stdin harness.

The four rules are already written and proven — they are the finder half of
**`wat-scripts/scratch-pad/probe-four-homes-census.wat`**, committed at `61dd04a3b`. Lift them
verbatim. They computed all ten replacements correctly against the live corpus:

```
:wat::core::Uuid/from-string       ->  :wat::uuid::from-string
:wat::core::Uuid/nil               ->  :wat::uuid::nil
:wat::core::Uuid/rfc4122-variant?  ->  :wat::uuid::rfc4122-variant?
:wat::core::Uuid/to-string         ->  :wat::uuid::to-string
:wat::core::Uuid/v4                ->  :wat::uuid::v4
:wat::core::Uuid/v5                ->  :wat::uuid::v5
:wat::core::Uuid/version           ->  :wat::uuid::version
:wat::core::List/of                ->  :wat::core::List
:wat::core::char/of                ->  :wat::core::char
:wat::core::regex::matches?        ->  :wat::regex::matches?
```

⚠ **Keep the keyword kind guard on every rule.** `Named` fires for string literals too, and a
literal's span covers its quotes while its `name` does not — splicing an unquoted name into that
span corrupts the literal into unquoted keyword syntax. Stone E's rider found that defect by adding
the guard the brief had omitted, across 1564 files. Measured for these ten names: the corpus has
**zero** genuine string-literal occurrences, so the guard changes no outcome here — keep it anyway,
because it is what makes the count honest as well as the rewrite safe.

⚠ **The path list is `*.wat` ONLY.** Run against a `.rs` file the finder returns zero matches,
silently — every Rust occurrence is inside a string literal, which the guard excludes. The header of
`rename-core-string-to-string.wat` shows `git ls-files '*.wat' '*.rs'` in its usage line; that line
is wrong and is itself the kind of pin this stone exists to remove. Do not copy it.

### Run it in this order

```bash
git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' > /tmp/paths.edn

# 1. FIND — expect exactly 239
./target/release/wat --grep ./wat-scripts/fixes/rename-four-families-to-their-homes.wat \
  < /tmp/paths.edn | wc -l

# 2. DRY RUN on a copy, and DIFF it. Never apply to the tree first.
rm -rf /tmp/dry && mkdir -p /tmp/dry && git archive HEAD | tar -x -C /tmp/dry
( cd /tmp/dry && git ls-files --version >/dev/null 2>&1 ; )   # /tmp/dry has no .git — feed it the same list
sed 's#"#"/tmp/dry/#' /tmp/paths.edn > /tmp/paths-dry.edn      # adapt the list to the copy
./target/release/wat ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths-dry.edn
diff -ru . /tmp/dry --exclude=.git --exclude=target | less     # read it; every hunk must be one name

# 3. APPLY to the tree
./target/release/wat ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths.edn

# 4. IDEMPOTENCE AS A QUERY — expect exactly 0
./target/release/wat --grep ./wat-scripts/fixes/rename-four-families-to-their-homes.wat \
  < /tmp/paths.edn | wc -l
```

Step 2's path-list adaptation is a sketch, not a recipe — build the `/tmp/dry` list whatever way
gives you a real byte-level diff of the copy against the tree. **The diff is the load-bearing step:**
every hunk must be exactly one of the ten names becoming exactly one of the ten targets. A hunk that
touches a quote, a paren, or a neighbouring token is a STOP.

Then hand-fix the **5 `.wat` comment lines** the rules cannot reach (a comment is not a node, so a
rule cannot touch one by construction):

```bash
grep -rn ';;.*:wat::core::\(Uuid/\|regex::\|List/of\|char/of\)' wat/ wat-scripts/ wat-tests/ tests/ --include='*.wat'
```

Leave `wat-scripts/scratch-pad/probe-four-homes-census.wat` alone. Its rules hold the OLD names as
string literals on purpose — post-migration it finds zero, and that is the negative control.

---

## Your own checks (cheap, targeted — the orchestrator runs the floor)

```bash
# every new codemod / touched .wat still parses and type-checks
./target/release/wat --check ./wat-scripts/fixes/rename-four-families-to-their-homes.wat

# the finder's count, before and after (239 -> 0)
```

A stdlib file under `wat/` cannot pass a standalone `--check` — `Privilege::Stdlib` comes from the
`STDLIB_FILES` pipeline, never a CLI target. `wat/fix.wat` fails it identically. That red is not
yours; do not chase it.

---

## Blast radius

`src/` (including `src/remedy/retirement.rs`), `crates/wat-reader/`, `crates/wat-doc/`, `tests/`,
`wat/`, `wat-tests/`, `wat-scripts/`,
`README.md`, `docs/USER-GUIDE.md`, and the one new file
`wat-scripts/fixes/rename-four-families-to-their-homes.wat`.

**No new directories.** `src/uuid/` and `src/regex/` are not part of this stone — the registry homes
`src/intrinsic/{uuid,regex,char,list}.rs` already exist and are correct. **No new types, no changed
handler bodies, no behaviour change of any kind.** If you find yourself editing logic rather than a
name, stop and read STOP-3.

---

## STOP triggers — each one means SHIP NOTHING and report

1. **STOP-1 — a diff hunk that is not exactly one name.** If the dry-run diff shows a changed quote,
   paren, or neighbouring token anywhere, the rule's span is wrong. Report the file, line, and the
   hunk verbatim. Do not "fix it up" by hand.
2. **STOP-2 — the finder count is not 239 before, or not 0 after.** Report both numbers and the
   command that produced them.
3. **STOP-3 — any door needs a behaviour change to accept the new name.** The claim of this stone is
   that all ten handlers are untouched. If a handler, a `TypeScheme`'s shape, or a match arm's logic
   has to change for the new name to work, that is a finding about the stone, not a task for you.
   Report which door and what it demanded.
4. **STOP-4 — `\c` stops round-tripping.** `(:wat::kernel::println \x)` must print `\x`. If it does
   not after Act 1, the parser/`to-wat`/`closure_extract` triple disagrees. Report all three sites'
   current text.
5. **STOP-5 — a name you were sent to change is already correct, or a room's line number does not
   hold what this brief says it holds.** The brief was written against `61dd04a3b`. Report the
   mismatch; do not widen the search on your own and do not assume the brief is right.
6. **STOP-6 — an old name still yields a bare `UnknownFunction` after you fed the table.** The
   target shape is a `MalformedForm` naming the replacement with a `:retirement` remedy. If an entry
   does not take effect, report the entry text and the actual error verbatim — do not work around it
   by leaving the old registration alive.

---

## Report back with

- The finder count before (expect 239) and after (expect 0), each with its command.
- The dry-run diff's shape: how many files, how many hunks, and confirmation that every hunk is one
  name. If anything else moved, that is STOP-1.
- **Every door you edited that this brief did not name**, with `file:line` — those are the ones the
  orchestrator's census missed, and they are the most valuable thing you can return.
- The old-name error for **one** verb of each of the four families, verbatim — it must be a
  `MalformedForm` naming the replacement, not a bare `UnknownFunction`.
- Anything that surprised you, including anywhere the brief was wrong.
- What you did NOT do and why.
