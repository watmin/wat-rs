# SCORE — the match arm is a bracket clause with a map pattern

No commit. Floor and clippy left to the orchestrator. Lands on H-2/H-2b/H-2c/H-3/J/K; those stones were not reverted.

**This stone is not closed.** The reader, the checker, the wat-fix, and most of the corpus moved. `wat/service.wat`'s generated `defservice` templates are STOP-4 (spliced `~@` binders and unit unquote heads). Stdlib freeze currently fails to parse `service.wat` (unexpected `]` at the serve-loop AllowPeer/DenyPeer arms). Rows 1–3 of the probe cannot run until that parses. Row 4 (retired paren clause refused) **passed** when the new reader was last built.

---

## Row 1 — bracket + map accepted

**blocked.** `a_map_pattern_arm_binds_its_declared_keys` ran with the new reader and got **exit 3**, empty stdout — not a match-arm failure. Freeze of stdlib died in `defservice` expansion (`wat/cache.wat:195`) because `wat/service.wat` templates were mid-rewrite. The probe fixtures themselves are already the target grammar.

## Row 2 — binding BY NAME

**blocked** on the same freeze. The reversed-keys fixture is on disk and un-ignored. It was not given a green run.

## Row 3 — unit arm is `{}`

**blocked** on the same freeze.

## Row 4 — ONE grammar, not two

**PASSED** (the only row that could run). Planted control
`tests/wat_lang/probe_arc109_match_arm__positional_control.wat` still has
`((:probe::Pair::Two a b) …)` and was **excluded from the codemod**. Exit ≠ 0.

## Row 5 — `#[ignore]`s gone

`grep -c '#\[ignore' tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs` = **0**.

## Row 6 — corpus moved BY CODEMOD

`wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` exists. Dry-run on `/tmp` copies of `sqlite.wat` / `cache.wat` / the positional control (the last excluded from the real run) showed the intended rewrite:

```
((:wat::core::Ok conn) …)     →  [:wat::core::Ok {:value conn} …]
(:wat::core::None body)       →  [:wat::core::None {} body]
((:probe::Pair::Two a b) …)   →  [:probe::Pair::Two {:a a :b b} …]
```

Applied to ~1867 `.wat` files (everything except the positional-control fixture and, after it rewrote itself, the wat-fix). Idempotent on a second pass of sqlite.

Unquote binders (`~assemble-p-sym`) first truncated to `~`; the wat-fix now reconstructs `~name` / `~@name` from the unquote child. Isolated dry-run of `core.wat` + `outcomes.wat` then produced `{:peer ~assemble-p-sym}`.

## Row 7 — `cond` untouched

Cond **clauses** were not rewritten. Files that also contain `match` show even-line diffs (match arms only): `wat/core.wat`, `wat/grep.wat`, `wat/rete.wat`, `wat/rete/compile.wat`, etc. STOP-5's `git diff --stat` over those files is **not** zero because match lives in the same files.

## Row 8 — wildcard and binding

`[_ body]` / `[<binder> body]` are 2-element vectors, head-discriminated. Not given a floor run.

## Row 9 — floor (ORCHESTRATOR)

Not run.

## Row 10 — clippy (ORCHESTRATOR)

Not run.

---

## What landed

### Rust reader (key-first, not `let`)

`src/match_arm.rs` — `parse_match_arm` / `parse_key_first_pairs` / `builtin_variant`. A List arm is refused with the retired-`(pattern body)` reason. A `{binder :field}` pair in a variant map is refused as let's destructure (STOP-1). Variant maps bind through `EnumValue.names` (G′), Option `{:value}`, Result `{:value}`/`{:error}`.

Wired in `eval_match`, `eval_match_tail`, `infer_match` / `detect_match_shape`, `normalize_match`, resolve walk, closure_extract, rete purity. The stepper fires `eval_match` once the scrutinee is canonical (no positional AST zip).

### wat-fix

Span-faithful paren-flip + pattern rewrite. Field names from a global defenum scan plus Option/Result seeds. Reader-macro lists (`~@arms`) are not treated as paren clauses. Generated FQDNs missing from the map fall back to binder names.

The wat-fix currently speaks the **old** match grammar so the pre-flip binary can run it. After rust lands it must itself be rewritten (it was, then restored so the stash-dance binary could still load it).

### `wat/service.wat` — STOP-4

`defservice` templates splice binders:

```
((~admin-init-kw ~@init-arg-names) (~init-name ~@init-arg-names))
(~admin-stop-kw body)
((~admin-allow-peer-kw pids) body)
```

A form-tree rewrite cannot invent a key-first map for `~@init-arg-names`. Hand-edit of the templates is in progress:

- `init-arg-map-ast` — a WatAST map built by `read-string` of `{:name name …}`
- Init/Resume/Stop/Hibernate/AllowPeer/DenyPeer arms in `dispatch-admin` and the serve loop converted to vectors
- `{ ~@init-arg-map-pairs }` **does not parse** (map splice is one form); replaced with `~init-arg-map-ast`

**Current freeze:** `#wat.parse/UnexpectedRBracket` at `wat/service.wat:1884` (AllowPeer serve-loop arm). The generated serve-loop still has leftover paren/bracket mismatch on the `do`+`match` body. Remaining generated arms (`(~reply-variant-kw resp)`, `(~op-variant-kw ~req-binder)`, SendOutcome/RecvOutcome/ServiceEvent inside the same quasiquotes) were going to be a second wat-fix pass after the templates parsed.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 routed through `let` | **held** in the reader. `parse_key_first_pairs` refuses Symbol keys. Hash-destructure of records stays binder-first as a 2-element vector (existing Open-shape; delimiter flip only). |
| STOP-2 binding by position | **not proven green.** The reversed-keys fixture is written. No successful run. |
| STOP-3 both grammars | Row 4 passed. Corpus is mixed only where `service.wat` templates are unfinished. |
| STOP-4 arm the codemod cannot rewrite | **fired.** Spliced `~@` binders and unit unquote heads in `defservice`. Reported above. Not left as a silent hand-edit of the 653-file corpus — the rest moved by wat-fix. |
| STOP-5 `cond` moves | **held** for cond clauses. Same-file match diffs exist. |

## Targeted checks

```
cargo test --release --test wat_lang probe_arc109_match_arm
  row 4 PASSED (retired paren refused)
  rows 1–3 FAILED exit=3 empty stdout — stdlib freeze, not the arm
./target/release/wat tests/wat_lang/probe_arc109_match_arm__declared_order.wat
  freeze error: defservice / service.wat (then cache.wat:195)
wat-fix dry-run /tmp sqlite+cache: intended rewrite
wat-fix full corpus: exit 0 (1867 files) after unquote-span fix
service.wat freeze: still UnexpectedRBracket :1884
```

## Files (this stone)

```
src/match_arm.rs                                      NEW — arm parser, key-first map
src/runtime.rs                                        eval_match / eval_match_tail / stepper
src/check.rs                                          infer_match / detect_match_shape / cover_variant_arm
src/resolve/{normalize,walk,mod}.rs                   vector arms
src/closure_extract.rs                                vector arms
src/rete/purity.rs                                    vector arms
src/lib.rs                                            mod match_arm
wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat NEW
wat/service.wat                                       templates mid-rewrite — NOT GREEN
wat/**, tests/**, wat-scripts/**, wat-tests/**        corpus via wat-fix
tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs  un-ignored
```

## What the next strike must do

1. Finish `wat/service.wat` serve-loop AllowPeer/DenyPeer `do` closers so the file parses.
2. Convert remaining generated unquote arms (`~reply-variant-kw`, `~op-variant-kw`, SendOutcome/RecvOutcome inside those quasiquotes) — either a second wat-fix pass that skips already-vector arms, or the same template treatment.
3. Rewrite the wat-fix's own match arms to the new grammar (stash-dance leftover).
4. `cargo test --release --test wat_lang probe_arc109_match_arm` — rows 1–3 must print `-1` / `-1` / `99`.
5. Floor / clippy stay with the orchestrator.
