# EXPECTATIONS — the rete vocabulary enters the registry

Written BEFORE the strike. Every bar derived from a measured number or a stated rule, never from
what I expect to see.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the registry grows by exactly 39 | `255-registry-census.wat` | **595** rows (556 + 37 rete + cond + reduce) |
| 2 | alias rows grow by 38 | same | **75** (37 + 37 rete + reduce; `cond` is a declaration, not an alias) |
| 3 | no rete row is left unregistered | the derived difference in the BRIEF | **0** |
| 4 | no alias restates an axis | `grep -c '@Purity' src/intrinsic/special/rete_alias.rs` | **0** |
| 5 | `reduce` still resolves AND answers | a probe calling `(:wat::core::reduce …)` | `foldl`'s answer |
| 6 | the wat `defalias` is gone | `grep -n 'defalias :wat::core::reduce' wat/seq.wat` | no match |
| 7 | the ledgers shrink by exactly the registered names | the ratchets' own red | names match rows added, both directions |
| 8 | the floor holds | orchestrator, centrally, once | 5139/5139 or better, 0 failed |
| 9 | clippy holds | `cargo clippy --release --all-targets -- -D warnings` | 0 |

Row 1's derivation: 556 measured this session by `255-registry-census.wat`; 37 unregistered rete
rows measured by the `255-b0-*` join; plus 2 orphan core targets. If row 1 lands on anything else,
one of those three numbers is wrong and THAT is the finding.

Row 4 is the honesty row. The design rejected *restriction* on the grounds that an inherited
`Partial` is true everywhere while a stamped `Total` is true only inside a `where`. If an axis has
to be restated to make a gate pass, the measurement was wrong — STOP-3.

## Independent prediction

**35–55 minutes.** Part 3 is 37 near-identical rows — bulk, not difficulty. Part 1 is one row with
a real doc comment. Part 2 is small and is where the time may actually go: STOP-2's proof is a
genuine unknown, not a formality.

## Trap doors — named before, not after

1. **`reduce` may become unresolvable (STOP-2).** The only claim that a registry alias dispatches
   is written about `:wat::rete::` rows; a `:wat::core::` name has not been shown to take that
   path. I believe it will and I have not proven it. If it does not, Part 2 depends on Phase 3a.
2. **`cond`'s row may collide with the macro table.** `cond` lives in `sym.macros`; a registry row
   adds a second place that knows the name exists. The design's claim is that these answer
   different questions (properties vs expansion), the same split the 51 handler-less rows already
   live with. If something dispatches `cond` at runtime because a row appeared, that is a finding.
3. **The 13 Unreviewed inheritances may trip a grading gate** that expects a registered row to
   carry a reviewed pole. If such a gate exists, it fires here and it is honest — the answer is to
   grade those core verbs, not to stamp the aliases.
4. **`@arg`/`@ret` copying is 37 chances to drift.** A wrong arity on an alias row is a lie the
   arity gate may or may not catch; row 3's derived list and the ledgers are the cross-check.

## What I will do on return

Re-run rows 1–7 myself before scoring. Rows 8 and 9 are mine alone and are the only verdict on
green. The rider's numbers are a hypothesis until a current `file:line` confirms them.
