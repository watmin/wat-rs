# SCORE — STONE 255.72: the wall has no exceptions — type the last two template constructors

**Executor: a Sonnet subagent.** Branch `main`, drawn against `c82daaa0a`. Not pushed.

## Site 1 — `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat`

**Typed** (no STOP). The untyped `(:wat::core::Tuple (first pair) (__work (second pair)))` at the
old line 75 is now:

```
out  (wat.type/Tuple :- [wat.type/i64 ~ret-ty] (:wat::core::first pair)
                        (:probe::__work (:wat::core::second pair)))
```

(now at line 81 — a 6-line explanatory comment was added above it). Exactly `wat/bracket.wat:491`'s
precedent: `~ret-ty` splices the raw type-AST node the probe already extracted off the reified
work-fn's declared return (`ret-ty (:wat::core::nth fn-ch 3)`, bound earlier in the same `let`) —
not re-derived by hand, not the `arg-t`/`ret-t` *text* built lower in the file for the Peer-type
keyword concat. The index slot is the literal `wat.type/i64` this runner's own `(i64, T)`
protocol always uses (the same literal `wat.type/i64` bracket.wat:491 uses for its own index slot).

### Behaviour-unchanged evidence (site 1)

This probe is **not wired into any Rust test** (`grep -rln probe-s3b-astsplice . --include=*.rs`
→ no hits); it is a standalone script run via `target/release/wat <file>`. I ran it both ways,
rebuilding the binary first to confirm it matched `HEAD` with no outstanding compile (`cargo build
--release --bin wat` → `Finished … in 0.05s`, confirming the binary already reflected the checked-out
source before any edit):

**Before** (`git show HEAD:wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` → `/tmp/before-s3b.wat`,
run through the up-to-date binary):

```
[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::core::keyword-node form: angle-bracket type parameters are illegal in a name (arc 109, \"annihilate the angle bracket\") — and that holds for a name BUILT at expand time exactly as it holds for one written in source: \":wat::kernel::Peer<(wat::core::i64,wat::core::i64),(wat::core::i64,wat::core::i64)>\". `:-` is the ONE parameterization operator. A macro must emit the type-application FORM `(Head :- [A B])`, not concatenate `Head` + \"<\" + args + \">\" into a keyword. A name is an atom; structure encoded inside one has to be re-parsed by every consumer, and that second parser is what this wall exists to make impossible.\" :location #wat.core/Span {:file \"/tmp/before-s3b.wat\" :line 59 :col 16 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 63 :col 61}}} :causes [] :head \":wat::core::keyword-node\" :reason \"angle-bracket type parameters are illegal in a name (arc 109, \"annihilate the angle bracket\") … \"}"}]
rc_before=1
```

**After** (edited file, same binary):

```
[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::core::keyword-node form: angle-bracket type parameters are illegal in a name (arc 109, \"annihilate the angle bracket\") … same message, byte-identical except :file is now the real path and :line/:col unchanged (59/16 … 63/61) \" ...}"}]
rc_after=1
```

Byte-for-byte identical failure, at the identical pre-existing site (lines 59–63: `peer-node`'s
construction via `keyword-node (string-concat ":wat::kernel::Peer<" … ">")`), both before and after
my edit. **This is a finding, not a cure I performed**: the probe already fails today, for a reason
the brief does not scope to this stone — `peer-node` (built a few lines above the site this brief
names) violates the *separate*, later arc-109 "annihilate the angle bracket" wall, which forbids
building a parameterized type name by string concatenation. That wall post-dates this probe and
nobody has re-run/fixed it since arc 109 landed. My typed-Tuple edit at line 81 is never even
reached at runtime — execution dies inside `peer-node`'s construction first — so I cannot show the
probe's documented "EXPECT 6 10" output; I can and did show my edit changes **nothing observable**
(identical crash, identical message, identical pre-existing location). Not STOP-2 (typing the site
did not change what it computes — nothing downstream of my edit ever runs, before or after).
Fixing the angle-bracket issue is a separate, larger repair outside this brief's named scope (it
is not the untyped-constructor wall, and the brief does not ask for it), so I left it alone and am
reporting it here instead of silently absorbing scope.

## Site 2 — `wat/rete/oracle/accum-pass.wat`

**Measured first, per the brief.** Can the oracle read a custom accumulator fold's declared
parameter type at run time?

**The runtime offers:** `:wat::runtime::signature-of-defn` (name/keyword → `Option<WatAST>`
signature head) + `:wat::runtime::extract-arg-types` (signature head → `Vector<WatAST>` of
declared param-type ASTs, verbatim). Both already used in production `wat/` code for the same
category of lookup (`wat/rete/compile.wat:833`, `wat/rete/oracle/stratify.wat:82`:
`(:wat::runtime::return-type-of head-fn)`).

**What it returns, measured against the oracle's own real fold** — I wrote
`wat-scripts/scratch-pad/probe-255-72-signature-of-defn.wat`, defining the EXACT fold the oracle's
own differential test uses (`tests/rete/probe_arc278_8custom_native_differential.rs`:
`:w::sum-of-squares [xs <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::i64`),
and ran it (`target/release/wat ./wat-scripts/scratch-pad/probe-255-72-signature-of-defn.wat`):

```
"n-args=1 first-arg-type-src=(wat.type/PersistentVector :- [wat.type/i64]) ty-kind=list ty-n-children=3 children= |symbol:wat.type/PersistentVector |keyword::- |vector:[wat.type/i64]"
"elem-ty-src=wat.type/i64"
rc=0
```

**Readable** — `signature-of-defn` + `extract-arg-types` returns the param type verbatim
(`(wat.type/PersistentVector :- [wat.type/i64])`); walking its children (`ast->children` →
`[symbol wat.type/PersistentVector, keyword :-, vector [wat.type/i64]]`) and the bracket vector's
own children yields the element type (`wat.type/i64`) exactly. **A1 applies; no STOP-1.**

One wrinkle the first attempt surfaced (kept, since the brief forbids silently discarding a
measurement): passing `acc-hd` (a *reified* `:wat::WatAST` value held in a variable) directly to
`signature-of-defn` failed —

```
oracle: "eval: #wat.runtime/TypeMismatch {:message \":wat::runtime::signature-of-defn: expected :wat::core::keyword or named function (e.g. :my::fn), got wat::WatAST `<WatAST>`\" :location #wat.core/Span {:file \"wat/rete/oracle/accum-pass.wat\" :line 141 :col 73 …} :op \":wat::runtime::signature-of-defn\" :expected \":wat::core::keyword or named function (e.g. :my::fn)\" :got {:type \"wat::WatAST\" :rendered \"<WatAST>\" :provenance nil}}"
```

— because `signature-of-defn` dispatches on whether its *argument expression* is a literal keyword
AST at the call site (it is a symbol reference here, `acc-hd`), and otherwise evaluates it and
requires the result to be a `Value::Keyword`/named-function value, not a reified `wat::WatAST`.
Fixed by round-tripping through the name string, the same `ast-name` + colon-strip +
`keyword::from-string` idiom already used elsewhere in this file (`var`) and in `wat/bracket.wat`:
`acc-kw (:wat::keyword::from-string (:wat::string::subs acc-nm 1 (:wat::string::length acc-nm)))`,
then `(:wat::runtime::signature-of-defn acc-kw)`.

**The fix** (`:else` branch, custom-fold dispatch): splices the fold's own declared element type
into the `PersistentVector` constructor bracket instead of leaving it bare:

```
call (:wat::core::quasiquote
       ((:wat::core::unquote acc-hd)
        (wat.type/PersistentVector :- [(:wat::core::unquote elem-ty)]
          (:wat::core::unquote-splicing vals))))
```

### Behaviour-unchanged evidence (site 2)

The oracle's own differential tests (`tests/rete/probe_arc278_8custom_native_differential.rs`),
run directly via `cargo test --release --test rete <name>` (not through `floor.sh`; no other
build/test ran concurrently with these):

**Before** my edit (baseline, same binary/source state apart from the uncommitted edit not yet
made):
```
test probe_arc278_8custom_native_differential::differential_custom_empty ... ok
test probe_arc278_8custom_native_differential::differential_custom_fold ... ok
test probe_arc278_8custom_native_differential::differential_custom_fold_value ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 521 filtered out; finished in 0.78s
```
```
test probe_arc278_8custom_native_differential::fence_rejects_impure_fold ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 523 filtered out; finished in 0.38s
```

**After** my edit (rebuilt — `cargo build --release` recompiled `wat` for the `.wat` source change,
`Finished … in 24.39s`):
```
test probe_arc278_8custom_native_differential::differential_custom_fold ... ok
test probe_arc278_8custom_native_differential::differential_custom_empty ... ok
test probe_arc278_8custom_native_differential::differential_custom_fold_value ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 521 filtered out; finished in 0.77s
```
```
test probe_arc278_8custom_native_differential::fence_rejects_impure_fold ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 523 filtered out; finished in 0.37s
```

Identical pass/fail set, identical assertion content (`fence_rejects_impure_fold`'s captured panic
message is byte-identical both runs: `"compile-condition: accumulator expr is not pure — ':wat::kernel::println' is not pure"`).
I also ran the full `rete` integration binary after the edit as a broader regression check (not a
substitute for the floor): `cargo test --release --test rete` → `test result: ok. 524 passed; 0
failed; 0 ignored; 0 measured; 0 filtered out; finished in 56.11s`.

## The wall's reach (gate 3)

Corpus-wide search for an untyped collection constructor, over every `git ls-files '*.wat'`
(excludes the gitignored `bootstrap/era/` tree, which is not part of the tracked corpus):

- The retired bare spelling `(:wat::core::{List,PersistentMap,PersistentVector,Tuple,Vector,HashMap,HashSet} …)`:
  **0 occurrences anywhere** in the tracked corpus (confirmed migrated).
- The `(wat.type/{…} …)` spelling **without** a `:- [...]` bracket: every candidate the regex
  surfaced was one of (a) inside a `;;` comment (prose discussing arc-109/251 type-*annotation*
  rendering, unrelated to this value-constructor wall — `tests/function/probe_stone255_70_*.wat:39`,
  `wat-scripts/fixes/one-param-spec.wat:353-354`, `wat-scripts/fixes/parametrics-take-a-type-vector.wat`,
  `wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat`, `wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat`),
  or (b) a bracketed function-parameter type **annotation** (`wat/fix.wat:1703/1707`,
  `acc <- (wat.type/Tuple :- […])`) that my regex's 40-char window missed because the bracket sits
  on the following line — not a value construction, and typed regardless.
- **0 bracket-less value-constructor sites outside `.wat.bad`/`.wat.golden`** — both named STOP-1
  sites this stone targeted are now typed; no other site exists.
- No code reads `TABLE-STONE-255.71-typed-constructors.edn` as a live allowlist
  (`grep -rln "TABLE-STONE-255.71\|stop1" tests/ src/` → no hits) — it is a documentation/census
  record, not an enforcement mechanism. It is the "named-exception list" the brief anticipated: it
  carried exactly 2 `:kind :stop1` rows (grep-counted, both at these two sites), now **0**
  (`grep -c ':kind :stop1' TABLE-STONE-255.71-typed-constructors.edn` → `0`) — both rows rewritten
  to `:kind :typed` with the actual spliced form and an updated `:reason`, and the stale "Exactly 5
  sites" header bullet corrected to describe the AMEND-2 → 255.72 history (5 → 2 → 0).

## Gates

| what | how | result |
|---|---|---|
| release floor | `scripts/floor.sh` (sole run, foreground, nothing else running) | `.floor/2026-10-01T00-50-25Z/clean.log`: `Summary [ 383.153s] 6235 tests run: 6235 passed (27 slow), 24 skipped` — matches the prior green floor's count exactly (`a34dc3406`: 6235 passed / 24 skipped) |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | `Finished `release` profile [optimized] target(s) in 11.83s`, rc 0 |
| the wall's reach | corpus-wide search (above) | **0** bracket-less sites outside `.wat.bad`/`.wat.golden` |

No reds were caused by this stone's own change (the floor ran green after both edits, in one pass,
with no re-run).

## Files touched

- `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` — site 1, hand-edited (a template site a
  codemod cannot reach, per `holon/CLAUDE.md`'s standing exception for `wat/bracket.wat:491`-class
  sites).
- `wat/rete/oracle/accum-pass.wat` — site 2, hand-edited (same reason).
- `wat-scripts/scratch-pad/probe-255-72-signature-of-defn.wat` — new scratch probe (kept per
  `wat-rs/CLAUDE.md`'s scratch-`.wat` convention: a durable, loadable, type-checked reference,
  swept by `every_wat_scripts_file_loads`), recording the site-2 measurement reproducibly.
- `docs/arc/2026/06/255-builtin-registry/TABLE-STONE-255.71-typed-constructors.edn` — the two
  `:stop1` rows converted to `:typed`; header bullet corrected.

## STOPs triggered

None. Neither STOP-1 (site 2's element type is readable) nor STOP-2 (neither site's typing changed
what it computes) fired.
