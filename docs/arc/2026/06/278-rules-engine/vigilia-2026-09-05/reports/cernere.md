# ward `cernere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I hold the spell. Report follows.

---

# `cernere` — wat-rs @ `21530efab` (`grok-rete`)

## L1 — defects

### L1-1 · The `:wat::` vocabulary is OPEN. No layer validates a leaf name; each defers to the other.

- `src/resolve/walk.rs:268` — `is_resolvable_call_head` opens with `if is_reserved_prefix(head) { return true; }`, and `RESERVED_PREFIXES` (`src/resolve/reserved.rs:14,25`) is `[":wat::", ":rust::", ":$bound::"]`. So **every** `:wat::…` call head resolves by prefix alone. Its own comment (`walk.rs:261-267`) says so: *"A wrong name under those prefixes (e.g. `:wat::holon::Bogus`) fails DOWNSTREAM at runtime or lowering… leaf-level validation is the type checker's concern."*
- `src/check.rs:5558` — the checker's terminal unregistered-scheme fallback opens with *"Resolve pass validated the name; we just don't have a scheme for it"*, and at `src/check.rs:5585` guards the UnknownCallee heuristic with `if args.len() == 1 && !k.starts_with(":wat::")` — i.e. `:wat::` heads are **explicitly exempted** and get a fresh, unconstrained type variable.

**Each comment cites the other layer as the one that checks.** Neither does. A phantom `:wat::` form passes parse, resolve, and type-check, and fails only if the call is actually reached at runtime.

The tree asserts this behaviour in two live green contracts:
- `tests/diagnostics/probe_arc241_stone10_remedy.rs:119-121` — `(:wat::core::xyzzy :T)` (`…_c04.wat:1`) must produce `"<startup succeeded — no error to display>"`.
- `tests/diagnostics/probe_arc241_stone10_remedy.rs:191-196` — same for `:wat::core::definitelywrong` (`…_c08.wat:1`).

**Consequence:** a misspelled, retired, or invented substrate verb is silently accepted at freeze in any position. Every "this file type-checks, therefore its names are live" claim in the repo — including `tests/lint/wat_scripts_fixes_load.rs`'s — is bounded to the `:wat::rete::` family, which is the only one with a textual resolver.

**Fix:** the terminal fallback must reject a `:wat::`-prefixed head with no scheme, no defclause, no intrinsic-registry row and no macro, through `CheckErrorKind::MalformedForm` + `remedy::remedies_for` (the machinery the retirement arms at `check.rs:4772-4850` already use). The `!k.starts_with(":wat::")` exemption at `5585` names its own escape hatch (`struct-new` for zero-field structs) — that is an allowlist, not a namespace.

---

### L1-2 · `:wat::kernel::` / `:wat::std::` get a *second*, earlier catch-all — and two live phantoms ride it.

`src/check.rs:4884-4909`:
```rust
_ if (k.starts_with(":wat::kernel::") || k.starts_with(":wat::std::"))
    && !k.starts_with(":wat::std::math::")
    && env.get(k).is_none()
    && env.get_defclause_clauses(k).is_none() =>
{
    // Unknown kernel / std path with no registered scheme or defclause —
    // accept and recurse.
```

Two phantoms live on it. Neither name exists anywhere in `src/`, `wat/`, or the rest of the tree (whole-tree `grep`, this session):

| site | phantom | real verb |
|---|---|---|
| `tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat:11`, `…_children_sig.wat:11`, `…_first_compose.wat:11`, `…_children_parametric.wat:16`, `…_first_err_empty.wat:9` | `:wat::kernel::abort` | `:wat::kernel::raise!` / `:wat::kernel::assertion-failed!` (`src/intrinsic/kernel/abort.rs` — `abort` is the *file*, not a verb) |
| `wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat:16` | `:wat::kernel::panic!` | same |

All five reflection fixtures are driven by `tests/reflection/wat_arc201_holon_ast_accessors.rs` via `startup_from_file`, and pass. In each the phantom sits in the **`:wat::core::None` / `Err` arm — the arm that exists to report the failure**. If `signature-of-defn` ever regresses, the diagnostic the fixture author wrote is not what fires; an `UnknownOp` at an unrelated head is.

The scratch-pad file is the sharper one: it is inside the corpus `tests/lint/wat_scripts_fixes_load.rs` walks, whose header claims parse+type-check makes rot unable to hide there. It cannot see this.

**Fix:** delete the `4884` arm (subsumed by the L1-1 cure); replace the six call sites.

---

### L1-3 · `examples/console-demo/wat/main.wat:29` uses `:wat::core::enum`, HARD CUT at Stone 241.9 — and a live substrate diagnostic points users at it.

`(:wat::core::enum :demo::Event …)` is rejected at check time by `src/check.rs:4795-4802` (`"'{}' is retired (Stone 241.9)"`). `console-demo` is a workspace **default-member** (`Cargo.toml:11,27`) whose `src/main.rs` is `wat::main! {}` — the wat is read at runtime, so the crate compiles green and nothing in the floor ever starts it. `cargo run -p console-demo`, the command in its own header, fails at startup.

It is not an orphan example. `src/check/error.rs:706` — a shipped `:wat::console::*` retirement message — ends: *"See examples/console-demo/wat/main.wat for the canonical ambient-stdio shape."* `src/distribution/mod.rs:85` cites it too. The substrate educates toward a file that does not load.

**Fix:** `enum` → `defenum` (a one-row `wat-fix` codemod, or by hand since it is one site); and the deeper cure is a gate that starts every `examples/**/wat/*.wat`, since `wat_scripts_fixes_load.rs` and `docs_wat_loads_or_declares_why_not.rs` between them leave `examples/` and `crates/` uncovered.

---

### L1-4 · Two rete heads that recorded codemods declare eliminated are still live in `tests/`.

`tests/rete/probe_constructor_meta_surface_total_enum.wat.bad:37-38`:
```
     fired   (:wat::rete::fire-rules-spec session)
     derived (:wat::rete::query-by-type-string fired "cg::Wrap")
```

- `:wat::rete::fire-rules-spec` — `wat-scripts/fixes/rete-oracle-sigil.wat:45` carries `;; rune:lint(rete-name-unminted) :wat::rete::fire-rules-spec — pre-\`$oracle\` spelling … retired by the very rewrite recorded below.`
- `:wat::rete::query-by-type-string` — `wat-scripts/fixes/type-query-to-defquery.wat:74` carries `… — the head this codemod detects and eliminates; retired by the migration recorded in this file, **so its absence is the tool working**.`

That rune's assertion is false in the tree. Two structural reasons it went unseen, and both are the finding:
1. `tests/lint/rete_names_in_wat_scripts_resolve.rs` walks **`wat-scripts/` only** (`PREFIX`/roots at its top) — a deliberate cut, but it means `tests/` (1060 `.wat`) and `wat-tests/` (88) have no rete-name resolver at all.
2. The extension is `.wat.bad`. `p.extension() == "wat"` is false for it, so every `*.wat` sweep — the load gate, and the `printf '["pathA" …]'` path lists the codemod doctrine prescribes — skips all **268** `.wat.bad` files in the tree.

The fixture happens to still fail for its declared reason (`probe_constructor_meta_surface_audit.rs:112-119` asserts `RhsArityMismatch` + `cg::gather` + `cg::Status::Active` + arity), so no test is currently lying. But `every_wat_bad_fixture_actually_fails.rs`'s own header states it *"cannot judge whether the REASON is true"* — and this file now carries two independent freeze-time failure causes.

**Fix:** re-run both recorded codemods with a path list that includes `.wat.bad`; extend the rete-name gate's roots to `tests/` + `wat-tests/` (both halves of its resolver already generalise).

---

### L1-5 · `tests/types/probe_arc256_generic_defclause_c05.wat:5` — the probe's own argument is a phantom, which voids the property it claims to prove.

```
(:wat::core::defclause :user::len-of ([v <- (:wat::core::Vector :- [T])] -> :wat::core::i64 0))
(:wat::core::defn :user::probe [] -> :wat::core::i64
  (:user::len-of (:wat::core::vector 1 2 3)))
```

`:wat::core::vector` (lowercase) exists nowhere in the tree — whole-tree grep returns this line and one prose mention of the *predicate* `vector?` at `src/runtime.rs:19283`. The constructor is `:wat::core::Vector` (`tests/types/ord_vec_i64_gt.wat:4`); `vec`/`list`/`tuple` were retired to it (`src/remedy/retirement.rs:113-124`).

`tests/types/probe_arc256_generic_defclause.rs:57-63` asserts `r.is_ok()` for the row its own header calls *"C05 parametric (Vector T) clause checks (RED at HEAD → GREEN; container-head + inner var)"*. Under L1-1 the argument's type is a **fresh unconstrained variable**, which unifies with `(Vector :- [T])` vacuously. The green cannot distinguish "container-head dispatch works" from "the argument had no type". The C03 guard row uses real types and does constrain the machinery — C05, the row that exists *specifically* for the parametric-container head, does not.

**Fix:** `(:wat::core::Vector :wat::core::i64 1 2 3)`. Then re-read whether C05 is still green.

---

## L2 — real weaknesses

### L2-1 · `crates/wat-edn/wat-edn-clj` — the cross-language type bridge reads a form the language hard-cut three arcs ago.

`crates/wat-edn/wat-edn-clj/src/wat_edn/scanner.clj:141,169` hard-code `(= "wat::core::struct" …)`. `:wat::core::struct` was HARD CUT at Stone 241.8 (`src/check.rs:4772`). Consequently the fixture `crates/wat-edn/wat-edn-clj/wat/shared.wat` — whose header (line 3-4) claims *"The same file would be consumed by wat-rs's type checker (as code) and by wat-edn-clj's load-types! (as schema)"* — is not wat any more. Every declaration in it is dead:

| line | form | status |
|---|---|---|
| 10 | `(:wat::core::use! :rust::wat_edn::write-str)` | no shim under `src/rust_deps/` (`cache`, `sqlite`, `custodia`, `marshal` only) |
| 12, 17, 24 | `(:wat::core::struct …)` | retired 241.8 → `defstruct` |
| 13, 18, 19, 26 | `(asset :Keyword)` | positional field grammar + bare `:Keyword` (canonical: `[asset <- :wat::core::Keyword]`) |
| 31 | `(:wat::core::define …)` | retired 241.11 → `defn` |

Both halves are tracked, and the Clojure test (`test/wat_edn/core_test.clj:15`) exercises the scanner against it, so the bridge is *green in Clojure while being unable to read any current wat file*. The header's claim is the wrong half: the code, not prose.

**Fix:** migrate `shared.wat` to `defstruct` + `[f <- :T]` and teach `scanner.clj` the current head; or, if the bridge is dead, say so in the crate rather than in a fixture that reads as live.

---

### L2-2 · `wat/deporder.wat:76` — `":wat::core::defprotocol"` is a phantom inside the frozen stdlib's def-head set.

`:wat::deporder::is-def-head?` builds a `HashSet` of declaration heads. `":wat::core::defprotocol"` is a member. Whole-tree grep: `defprotocol` appears in **zero** `src/` files. Its siblings all exist (`structtype`, `newtype`, `recordtype` all attest in `src/`). The only other trace is three orphaned fixtures — `tests/types/probe_diagnostic_defprotocol_dispatch_p{1,2,3}.wat`, which have no `.rs` driver and emulate protocol dispatch by hand.

It is a string, so it fails open — the arm simply never matches. But `deporder` is what enforces stdlib load order, and a def-head it cannot recognise is a def-site it never records. The entry advertises a form the language does not have.

**Fix:** drop the row (and either drive or delete the three orphan fixtures).

---

### L2-3 · `wat/spawn.wat:235-236` — two `derive` edges to a marker type the code retired, kept alive because `derive` never resolves its arguments.

```
(:wat::core::derive :wat::kernel::Thread  :wat::spawn::Spawned)
(:wat::core::derive :wat::kernel::Process :wat::spawn::Spawned)
```

`:wat::spawn::Spawned` is declared nowhere. `src/types.rs:3877-3901` (`":wat::core::derive"`) takes both operands as raw `WatAST::Keyword` strings and calls `register_subtype` — no declaration lookup on either side. So the edges register against a name that does not exist.

And the file says so itself, 60 lines down: `wat/spawn.wat:296` — *"arc 291 3a-ii-β: handle is the lineage PEER (`(Peer' :- [Sh Lu])`), **no longer the opaque :Spawned marker**"* — with `Launched.handle <- (:wat::kernel::Peer :- [Sh Lu])` at line 301. The header comment at `wat/spawn.wat:232` still asserts the retired role.

**Fix:** delete both `derive` lines and the 231-232 header block, or declare `Spawned` and type the field by it. Separately: `register_subtype` accepting an undeclared parent is the same open-vocabulary root as L1-1, one form down.

---

### L2-4 · `src/resolve/mod.rs:25` documents a check the resolver does not perform.

The module doc lists what a call head must resolve to, beginning: *"A known `:wat::core::*` language form (defn, fn, let, if, the builtin arithmetic / comparison / boolean ops, …)"*, and closes at line 51: *"Anything else is an unresolved reference and halts startup with a clear error citing the offending path."*

`walk.rs:268` returns `true` for every `:wat::`-prefixed head before any of that is consulted. Lines 40-41 are honest about `:wat::kernel::` / `:wat::std::` (*"accepted here"*); line 25 is not honest about `:wat::core::`, and line 51's "anything else" is false for the entire reserved root. This doc is inside `src/` and it is the reason `check.rs:5558` believes what it believes.

**Fix:** restate 25 and 51 as "any `:wat::`-prefixed path is accepted by prefix; leaf validation is UNOWNED" — or, better, make line 25 true and close L1-1 here rather than in `check.rs`.

---

### L2-5 · `tests/types/typed_if_match_bare_symbol_variant.wat.bad:15,20` — `:wat::core::panic!` is not a form.

Whole-tree grep finds `:wat::core::panic!` in `docs/` prose only — never in `src/` or `wat/`. The `.bad` fixture's declared subject is the bare-symbol variant hint, and `tests/types/typed_if_match.rs:210-219` asserts substance with two `assert_malformed_mentioning` calls, so the test is not currently lying. But the assertion is an `.any()` over the error vector, so an additional cause is invisible, and the fixture is one edit away from passing for the wrong reason. Same class as L1-4.

**Fix:** `:wat::kernel::raise!`.

---

## L3 — judgement

- **The `.wat.bad` extension is a silent scope hole in every tool this repo owns.** `wat_scripts_fixes_load.rs` filters `extension == "wat"`; the codemod doctrine in `CLAUDE.md` prescribes explicit path lists that in practice come from `*.wat` globs; `every_wat_bad_fixture_actually_fails.rs` is the only thing that reads the 268 of them, and it reads them for *one* property. A negative fixture is still code, and 268 files that no migration reaches will drift into a museum of the vocabulary at the moment each was written. `fixture.bad.wat` would put them back inside every existing sweep for the cost of one rename.
- **`tests/lint/rete_names_in_wat_scripts_resolve.rs` is the right instrument aimed at 27% of the corpus.** Its three-source resolver (registry / attestation / declaration), and its measured proof that no two of the three are a subset, is the best-argued gate I read this session. Its roots are `wat-scripts/`. `tests/` holds 1060 `.wat` and `wat-tests/` 88 — and L1-4 sits in `tests/`. Widening the roots costs nothing the design has not already paid for.
- **`tests/wat_lang/wat_arc143_manipulation.wat:26` renames `:wat::core::foldl` → `:wat::list::reduce`.** Name-as-data, so not a phantom form — but `:wat::list::` was retired to `:wat::seq::` (`wat-scripts/fixes/rename-list-to-seq.wat`) and `:wat::seq::` was itself retired at 118.2a (`wat/seq.wat:3-9`). The golden bakes a doubly-dead namespace as the exemplar of what a rename produces.

---

## What I could not check

- **I did not build or run anything.** Every "the checker accepts X" claim rests on two independent readings — the source at `resolve/walk.rs:268` + `check.rs:5585`, and live green tests that assert `is_ok()` / `"<startup succeeded>"` on fixtures containing the phantom. I did **not** drive a fresh phantom through `startup_from_file` myself. If you want L1-1 hardened before acting, that is the one-line probe.
- **My Rust-side sweep for phantom wat names in diagnostic strings was vacuous and I discarded it.** wat names appear in `.rs` files *only* inside string literals, so the "attested outside a string" universe I built was empty and 425 of 425 names read as unresolved. I cannot tell you whether other substrate error messages name forms that no longer exist. `src/check/error.rs:706` (L1-3) was found by a different route — grepping for the example path.
- **My `.wat` corpus resolver is generous in one direction I could not close.** A name counts as attested if it appears *anywhere* in `src/` or `crates/` — comments included. A form mentioned only in a Rust comment and never registered reads as live. `:wat::kernel::abort` survived only because it appears in *no* Rust text at all; a phantom with a stale comment somewhere in `src/` would have escaped me.
- **Bare-symbol (non-keyword) call heads: not checked, by me or by the substrate.** `src/resolve/mod.rs:53-57` states the resolve pass does not check lexical references and defers to runtime `UnboundSymbol`. My sweep only reads `:`-prefixed tokens. That surface is entirely unexamined.
- **Macro expansions.** I treated quasiquote templates as data per the spell. A phantom head that appears *only* in a macro's emitted form — never written literally in the corpus — is invisible to this sweep.
- **`docs/**/*.wat`** excluded per the builder's ruling. Note that `docs/arc/2026/05/130-.../complected-2026-05-02/substrate.wat` carries retired `struct`/`enum`/`core::panic!` throughout; I did not count it and no finding above rests on it.
- **`tools/*.wat`** (6 files) are gitignored (`.gitignore:9`) — codemod dry-run output, out of scope.
- **The Clojure half of L2-2** — I read `scanner.clj` and `core.clj` but did not run `clj -M:test`, so I cannot say whether that suite is currently green.
