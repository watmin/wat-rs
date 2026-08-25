# BRIEF — STONE: the four that got homes they had not earned (phase 1, the ten verbs)

> **REFRESHED 2026-08-25 against `c5f1ee487`.** The first draft of this brief was written before
> arc 300 stone D, arc 278's wat-grep stone, and arc 282's `fix-text-apply` wall. All three changed
> what this stone has to do. **Every number below was re-derived after them**; nothing was carried
> forward from the earlier draft.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-four-that-got-homes-they-had-not-earned.md`
— read its `⛔ GROUNDED` and `⛔ AMENDED` sections; everything above them is a superseded draw.

---

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched.

You make text edits and run the cheap checks named below. The orchestrator builds, floors, clippies
and commits.

⚠ A stdlib `.wat` edit is invisible until a rebuild; you are not editing any. A file under `wat/`
cannot pass a standalone `--check` — not your red.

---

## The work in one paragraph

Ten Rust-implemented verbs are registered under `:wat::core::` and none belongs there. Move each to
the namespace it earns. **No handler body changes** — that is what makes it one stone instead of
four. Two mechanisms: the `.wat` corpus moves by a codemod **that already exists and is already
correct**, and the `.rs` side moves by hand because in Rust these names live inside string literals.

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

`List/of` and `char/of` are **finishing** a migration: every other collection type is already its own
constructor. `(:wat::core::PersistentVector 1 2 3)` evaluates today; `(:wat::core::List 1 2 3)` and
`(:wat::core::char "x")` are `UnknownFunction`. Both names already exist as TYPES and already pass
`--check` in annotation position — verified. `:wat::` is reserved at the root
(`src/resolve/reserved.rs:14`), so `:wat::uuid::` and `:wat::regex::` are substrate-owned the moment
they are written; there is no namespace registry to update.

---

## ★ WHAT THREE LATER STONES ALREADY DID FOR YOU

**Arc 300 stone D removed two of the three worst doors.** The earlier draft's sharpest hazard was
that `\c` desugared at PARSE TIME into `(:wat::core::char/of "x")`, so every `\c` in the corpus
became a call to a name no corpus file contained. `WatAST::CharLit` killed that. Verified at HEAD:

- `crates/wat-reader/src/parser.rs:399,402` — **prose only now.** Not an emitter.
- `src/runtime.rs:21386` — **prose only now.** `to-wat` renders a `CharLit` directly.

**Arc 278 made the census honest.** wat-grep used to swallow an unreadable file into an empty fact
base; the corpus number was measured on an instrument with an unknowable denominator. It now reports
and exits non-zero, and three unreadable `.wat` files were renamed `.wat.bad`. The 190 below was
measured with **exit 0 and empty stderr**.

**Arc 282 put a wall under the codemod.** `fix-text-apply` now compares the edit's claimed old-text
against the source and REFUSES on disagreement. So if any phantom match survives anything, **the
codemod raises instead of corrupting.** You have a safety net the first draft did not.

---

## ACT 1 — the `.rs` rename (134 occurrences)

### The registrations — change the string in the attribute, and nothing else

| room | lines |
|---|---|
| `src/intrinsic/uuid.rs` | 40 53 54 55 64 78 79 80 88 116 129 130 131 138 159 168 169 170 177 193 201 202 211 222 223 230 246 258 259 266 |
| `src/intrinsic/regex.rs` | 14 29 30 38 |
| `src/intrinsic/list.rs` | 1 4 18 31 32 |
| `src/intrinsic/char.rs` | 13 28 29 36 |

Each is a `#[wat_intrinsic("…")]`, a `const OP:`, a `///` line, or an `@example`.

### ★ THE RETIREMENT TABLE — the substrate's memory of its own rename

`src/remedy/retirement.rs` maps a retired form to its replacement so the error at an old name
**names the new one**. Its header states the rule: *"Each HARD CUT stone appends its retirement entry
at the arc's ship time."* **Append exactly ten**, and add the matching rows to the header's
`## Arc history` table. There are **25** entries at HEAD; there must be **35**.

Measured, so you know the target shape — `(:wat::core::Char "x")`, retired by stone 242.1, today:

```
MalformedForm  "':wat::core::Char' is retired (Stone 242.1); use ':wat::core::char' instead"
:remedies [#wat.kernel/Remedy {:form ":wat::core::char" :kind :retirement …}]
```

A bare `UnknownFunction` on an old name means the table was not fed.

⚠ Do **not** add entries for `:wat::core::string::*`. Stone E moved those and left the table unfed;
that is a separate finding on E's ledger, out of this stone's scope.

### The remaining EMITTER — it constructs the call rather than making it

`src/closure_extract.rs:1998` (`Uuid/from-string`) and `:2015` (`List/of`) encode portable values
into a wire format by **building a call**. No corpus file contains those call sites. `:2011` is
prose. This is the door class a call-site census cannot see; stone D removed its two siblings.

### The type-checker, the purity classifier, the rest

| room | lines | why |
|---|---|---|
| `src/check.rs` | 17598 17620 17632 17643 17653 17663 17673 17683 17695 17699 17703 | `register_builtins` — one `env.register` `TypeScheme` per verb; where the new name acquires a type |
| `src/check.rs` | 3017 3018 3020 · 15289 15291 15311 | `infer_list`'s special-cased `List/of` arm and `infer_list_of`'s own `callee:` string |
| `src/rete/purity.rs` | 10 250 489 490 491 492 551 563 1603 1604 1610 1615 | classification arms + a unit test. ⚠ `:wat::core::List?` and `List/length` at 551 are **NOT in scope** — only `/of` moves |
| `src/rete/purity.rs` | 2215 | a frozen **alphabetical** name list (a ratchet). `:wat::core::char` occupies the slot `char/of` held |
| `src/rete/purity.rs` | (prefix door) | `head.starts_with(":wat::core::regex::")` — give it `:wat::regex::` the way stone E gave it `:wat::string::` |
| `src/value/value.rs` | 311 339 | doc comments naming the constructors |
| `src/string/mod.rs` · `crates/wat-doc/src/lib.rs` | 35 · 1062 | prose / a doc-parser test fixture string |
| `crates/wat-reader/src/ast.rs` | 86 90 | `CharLit`'s doc comment naming the surviving verb |

### The tests

`tests/value/wat_arc220_char.rs` (11 12 13 14 58 80 106 163) ·
`tests/program/probe_arc213_program_edn_roundtrip.rs` (16 26 110 144 163 167) ·
`tests/collection/list.rs` (4 32 39 54) · `tests/types/uuid.rs` (68 80 135) ·
`tests/rete/probe_fence_names_the_head.rs` (15 **87**) ·
`tests/rete/probe_arc278_seq1b_list_hofs.rs` (56 135) · `tests/rete/probe_arc278_6a_purity.rs` (5) ·
`tests/program/wat_arc278_sigma_fn_purity_gate.rs` (15) · `tests/kernel/wat_string_ops.rs` (95) ·
`tests/collection/probe_seq_container_parity.rs` (70)

⚠ `probe_fence_names_the_head.rs:87` pins an **exact error-message string** containing
`':wat::core::Uuid/v4'`. That assertion moves with the name.

### ⛔ ONE FILE WHERE SOME REFERENCES MOVE AND SOME MUST NOT

`tests/wat_lang/gate_char_literal_is_a_literal.rs` — arc 300 stone D's gate. Six references, and
they split:

```
 4, 11, 54   PROSE describing the RETIRED desugar — "the parser previously desugared \a into
             (:wat::core::char/of "a")".  HISTORICALLY TRUE. It emitted `char/of`, not `char`.
             ⛔ DO NOT CHANGE THESE.  (FM 14 bucket C — a retirement record.)

95, 101, 102 THE LIVE VERB — the doc line, `const HEAD: &str`, and the parsed input of
             `the_char_of_verb_still_parses_as_an_ordinary_call`.  ⛔ THESE MUST CHANGE.
```

Getting this backwards either breaks a passing test or falsifies a historical record. Read each of
the six before touching any.

### The four files neither glob reaches

```
tests/cli/wat_mcp__persist_def_value.jsonl                                :  one Uuid/v4  (a golden — recapture, do not hand-edit if a runner regenerates it)
tests/program/wat_arc278_sigma_fn_purity_gate_presence_nondeterministic.wat.bad :  Uuid/v4
tests/cli/wat_grep__malformed.wat.bad                                     :  one
docs/USER-GUIDE.md (15) · README.md (1)                                   :  user-facing prose
```

`README.md`'s line also names the deleted `string_ops`; fix that half while you are there.

### KEEP — do not touch

- The **four retirement-record comments** naming `:wat::core::Char/of` (`parser.rs:402`,
  `check.rs`, `closure_extract.rs:2001`, `runtime.rs`). They record stone 242.1; history, not code.
- Everything under `docs/arc/**` except this arc's own DESIGN/BRIEF/EXPECTATIONS.
- **`wat-scripts/scratch-pad/probe-four-homes-census.wat`** — its rules hold the OLD names as string
  literals on purpose. Post-migration it must find **0**; that is the negative control.

The one `Char/of` that DOES move is prose: `wat-tests/holon/char-round-trip.wat:3`.

---

## ACT 2 — run the codemod. It already exists and is already correct.

**`wat-scripts/fixes/rename-four-families-to-their-homes.wat`** is written, `--check`-clean, on arc
282's new edit API, and its finder already returns exactly the census number. **Do not rewrite it.**
Read it, satisfy yourself, and run it.

```bash
git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' > /tmp/paths.edn

# 1. FIND — expect 190, exit 0, EMPTY stderr.
./target/release/wat --grep ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths.edn | wc -l

# 2. DRY RUN on a copy and DIFF it. Never the tree first.
rm -rf /tmp/dry && mkdir -p /tmp/dry && git archive HEAD | tar -x -C /tmp/dry
#   build the path list against /tmp/dry however gives you a real byte-level diff, then:
./target/release/wat ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths-dry.edn
diff -ru . /tmp/dry --exclude=.git --exclude=target --exclude=.floor

# 3. APPLY
./target/release/wat ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths.edn

# 4. IDEMPOTENCE AS A QUERY — expect 0
./target/release/wat --grep ./wat-scripts/fixes/rename-four-families-to-their-homes.wat < /tmp/paths.edn | wc -l
```

**The diff is the load-bearing step.** Every hunk must be exactly one of the ten names becoming
exactly one of the ten targets.

★ **If the codemod RAISES, read the message before anything else.** Arc 282's wall means a raise is
`fix-text-apply` telling you the rule's claim and the source disagree at a named offset — that is a
real find about a phantom match, not a bug to route around. It is STOP-2.

Then hand-fix the `.wat` **comment** lines the rules cannot reach (a comment is not a node):

```bash
grep -rn ';;.*:wat::core::\(Uuid/\|regex::\|List/of\|char/of\)' wat/ wat-scripts/ wat-tests/ tests/ --include='*.wat'
```

---

## Blast radius

`src/`, `crates/`, `tests/`, `wat/`, `wat-tests/`, `wat-scripts/`, `README.md`,
`docs/USER-GUIDE.md`. **No new directories** — `src/intrinsic/{uuid,regex,char,list}.rs` already
exist and are the correct registry homes. No new types, no changed handler bodies, no behaviour
change of any kind.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — a diff hunk that is not exactly one name.** Report the file, line, and hunk verbatim.
   Do not fix it up by hand.
2. **STOP-2 — the codemod raises.** Paste `fix-text-apply`'s message whole: it names the offset, the
   claim, and what is actually there. That is a finding, not an obstacle.
3. **STOP-3 — the finder count is not 190 before, or not 0 after.** Report both, with commands.
4. **STOP-4 — a door needs a behaviour change to accept the new name.** The claim of this stone is
   that all ten handlers are untouched. Report which door and what it demanded.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against
   `c5f1ee487`. Report the mismatch; do not widen the search on your own.

---

## Your own checks

```bash
./target/release/wat --check ./wat-scripts/fixes/rename-four-families-to-their-homes.wat
# the finder count, before and after (190 -> 0)
grep -c 'RetirementEntry' src/remedy/retirement.rs        # 25 at HEAD; must be 35
```

## Report back with

- The finder count before (expect **190**) and after (expect **0**), each with its command, and
  confirmation that stderr stayed empty and the exit was 0.
- The dry-run diff shape: files, hunks, and confirmation every hunk is one name.
- The old-name error for **one** verb of each of the four families, **verbatim** — each must be a
  `MalformedForm` naming its replacement, not a bare `UnknownFunction`.
- **Every door you edited that this brief did not name**, with `file:line`. The orchestrator's
  census has been wrong about this corpus repeatedly today; these are the most valuable thing you
  can bring back.
- How you dispositioned each of the six references in `gate_char_literal_is_a_literal.rs`.
- Anything the brief got wrong.
- What you did NOT do, and why.
