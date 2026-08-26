# STONE — the header file that never compiled: wat-edn's cross-language schema

DRAWN + BRIEFED 2026-08-25, against `f6ead6e9b` + the uncommitted ungated-corpora work in the tree.

## Why this stone exists

The previous stone's rider fired **STOP-1** on `crates/wat-edn/wat-edn-clj/wat/shared.wat` and
shipped nothing for it. **That was the right call** and this stone is its resolution.

The file states its own contract in its header, lines 3–5:

> *"Acts as a HEADER FILE for cross-language type sharing. The same file **would be** consumed by
> wat-rs's type checker (as code) and by wat-edn-clj's `load-types!` (as schema)."*

**"Would be."** The dual contract is aspirational and has never held. The file does not type-check —
it uses `:wat::core::struct` (retired, Stone 241.8) and `:wat::core::define` (retired, Stone 241.11),
so wat-rs's checker has never consumed it. Only the Clojure half was ever real.

**This stone makes the sentence true for the first time, and then makes it stay true** — the
ungated-corpora wall (`tests/lint/every_ungated_wat_checks.rs`, already in the tree) walks this file,
so once it checks, it can never silently stop checking.

## ⛔ WHY THE PREVIOUS RIDER COULD NOT JUST FIX THE `.wat`

`crates/wat-edn/wat-edn-clj/src/wat_edn/scanner.clj` **hand-parses the retired grammar as literal
text.** Verified on disk:

- `:141` and `:169` compare against the literal string `"wat::core::struct"`.
- `:136` documents the layout `( :wat::core::struct :path::Name (field :Type) ...)`.
- `extract-structs` **silently skips** any form whose head does not match — it does not throw.

So migrating the `.wat` alone makes `extract-structs` return `[]`: **zero types loaded, no error**,
and every Clojure consumer of `load-types!` breaks in silence. The wat and the Clojure parser are one
unit and must move together. **That is the whole work of this stone.**

## The measured ground

```
clojure -M:test  (in crates/wat-edn/wat-edn-clj)   ->  39 tests, 96 assertions, 0 failures, 0 errors
./target/release/wat --check <shared.wat>          ->  exit 1
grep -c struct  scanner.clj / scanner_test.clj     ->  14 / 27
wc -l           scanner.clj / scanner_test.clj     ->  181 / 92
```

The Clojure toolchain works here and its deps resolve. **That is why nothing about this is deferred.**

---

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first. **Ending your turn ENDS you** — nothing
will wake you; there is no notification coming. Every command **FOREGROUND**, blocking.
**You may not spawn sub-agents.**

Do not commit, push, stash, revert, or `git checkout`. `git stash@{0}` must never be touched.

⚠ **The tree already holds another stone's uncommitted work** (console-demo fixed + gated, a deleted
corpse, a new wall in `tests/lint/`). **Leave all of it alone.** Your only files are the four named
below plus whatever docs carry the grammar.

You may run `./target/release/wat --check <f>`, `cargo build --release`,
`cargo test --release --test lint every_ungated_wat_file_checks`, and
`clojure -M:test` from `crates/wat-edn/wat-edn-clj`. **Not** the full floor, **not** clippy — the
orchestrator measures those centrally once the tree is quiescent.

---

## THE WORK

### PART 1 — `crates/wat-edn/wat-edn-clj/wat/shared.wat` → the current language

Derive each shape from a LIVE site; do not invent one.

- **`:wat::core::struct` → `:wat::core::defstruct`**, fields become **binder vectors**.
  Canonical live reference: `wat/cache.wat:272-274`, `wat/holon.wat:106-107`. Shape:
  `(:wat::core::defstruct :fq::Name [field <- :Type  field2 <- :Type2])`.
  **`defstruct` takes NO purity marker** — that is `defenum`'s rule, not this one. Confirm against
  the live sites rather than assuming either way.
- **`:wat::core::define` → the current fn form.** The retirement remedy names `:wat::core::defn`;
  the *signature shape* also changed. Derive it from a live `defn` with a typed param and a return
  type. The remedy names the new NAME, not the new SHAPE — that trap already cost this arc one
  round on `enum`→`defenum` (four changes, one named).
- **`:wat::core::format`** — `"[%s] %s @ %f"` positional is not the current form. Derive the current
  spelling from a live call site.
- **`:Keyword` (lines 13, 18, 19, 26)** is the one genuine unknown I did not resolve for you: it is
  not fully-qualified and I found no `:wat::core::Keyword` in `wat/`. Find what the keyword type is
  actually called and use it. If there is no such type, that is a finding — report it.
- **`:wat::time::Instant` (line 22) is REAL** — a builtin available at startup (`wat/program.wat:7`).
  Leave it.
- **Line 10, `(:wat::core::use! :rust::wat_edn::write-str)`** — nothing in the file calls
  `write-str`. Check that claim yourself; if nothing calls it, delete the line and say so. If
  something does, it must resolve or the file cannot check.
- **Preserve the fixture's INTENT.** Line 30's comment — *"A function definition — should be ignored
  by the scanner"* — is the file's test content: it proves the scanner skips non-struct forms. Keep
  a function definition there, in current spelling. Change the language, never the demonstration.

### PART 2 — `src/wat_edn/scanner.clj` → teach it the new grammar

Match on the `defstruct` head and parse **binder-vector** fields (`[name <- :Type …]`) instead of
parenthesized pairs. Keep `extract-structs`'s skip-don't-throw behaviour for non-matching heads —
that is deliberate and Part 1 line 30 depends on it.

⚠ **The silent-`[]` failure mode is the defect to design out.** A scanner that finds zero structs in
a file that plainly contains three should not return `[]` quietly. If you can make that
state loud without disturbing the skip-unknown-forms contract, do — and say what you chose and why.

### PART 3 — `test/wat_edn/scanner_test.clj` → 27 sites

The tests carry the grammar in their fixture strings. Migrate them with the scanner.
**BAR: `clojure -M:test` returns 39 tests / 96 assertions / 0 failures / 0 errors — the same
numbers as the baseline above.** A test count that DROPS means you deleted coverage.

### PART 4 — the docs that teach the grammar

`crates/wat-edn/docs/IPC-BRIDGE.md` (:286, :357), `crates/wat-edn/docs/USER-GUIDE.md` (:551, :644),
`crates/wat-edn/interop-tests/README.md` (:35), and `wat-edn-clj/README.md` if it shows a struct
form. A doc that teaches the retired spelling is the same lie in a friendlier voice.

### PART 5 — the header sentence

`shared.wat`'s line 3–5 says the checker **"would be"** a consumer. After this stone it **is** one,
and a wall enforces it. Rewrite those lines to state the contract in the present tense and name the
wall that holds it.

---

## STOP triggers — each rejects; none permits a lesser delivery

1. **STOP-1 — no keyword type exists for `:Keyword`.** Report what you searched and what you found;
   ship nothing rather than invent a type name.
2. **STOP-2 — the Clojure test count changes from 39/96.** A different count means coverage moved.
   Report the delta and which test changed; do not adjust the numbers to match.
3. **STOP-3 — `shared.wat` reaches `--check` exit 0 only by removing one of the three structs or the
   function definition.** The file's content IS the fixture. Report the conflict.
4. **STOP-4 — a room's line number does not hold what this brief says.**

---

## Acceptance — every row derives its bar

```bash
# 1. the header file finally compiles.            BAR: 0
./target/release/wat --check crates/wat-edn/wat-edn-clj/wat/shared.wat; echo "EXIT=$?"

# 2. the Clojure half still passes, undiminished. BAR: 39 tests / 96 assertions / 0 / 0
cd crates/wat-edn/wat-edn-clj && clojure -M:test; cd -

# 3. the whole ungated corpus is green — the wall in the tree.
cargo test --release --test lint every_ungated_wat_file_checks

# 4. no retired spelling survives anywhere in the crate.  BAR: 0
git grep -c 'wat::core::struct\|wat::core::define' -- crates/wat-edn/ | wc -l
```

## Report back with

- Each command's **actual output**, naming the command that produced each number.
- **The `.wat` before/after in full** — it is 37 lines; show it.
- What you chose for the silent-`[]` failure mode in Part 2, and why.
- The `:Keyword` resolution: what you searched, what you found, what you used.
- Anything the brief got wrong.
- What you did NOT do, and why.
