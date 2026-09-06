# EXPECTATIONS — STONE: a keyword is a keyword

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **a slash-in-name keyword renders as a KEYWORD** | `:wat::rete::Explained/support` → **`:wat.rete.Explained/support`**. Not `#wat.ast/Keyword {:path …}` |
| 3 | ★★★ **AND IT READS BACK AS THE SAME WAT KEYWORD** | decode of `:wat.rete.Explained/support` yields `:wat::rete::Explained/support`. **Row 2 without row 3 trades a lossless ugly carrier for a lossy pretty one — strictly worse than today** |
| 4 | ★★★ **the doc row loses all seven** | `pprintln` of the step-payload Row: `grep -c '#wat.ast/Keyword'` = **0**, and `(:wat.rete.Explained/support ex)` appears as a call |
| 5 | ★★★ **the verbatim carrier still EXISTS and still fires** | a keyword EDN genuinely cannot spell still becomes `#wat.ast/Keyword`. **Deleting the fallback passes rows 2-4 and reintroduces the silent-type-change its own doc records** |
| 6 | ★★★ **the change is CONFINED to the slash-in-name shape** | a plain `:wat::core::first` still renders `:wat.core/first`; a no-`::` keyword is unchanged. **Every other keyword byte-identical** |
| 7 | ★★ the corpus formatter is unmoved | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` |
| 8 | ★★★ **every golden that moves is EXPECTED and enumerated** | list each `.edn` golden whose bytes change, and say why each is the `X/y` shape. **An unexplained golden diff is an unrelated regression hiding in an expected one** |
| 9 | ★★ the doctest gate still passes | 89 corpus `@example` lines carry this shape; the gate runs them |
| 10 | ★★ negative control, committed | revert the split → row 4's count returns to 7 and the round-trip test goes RED |
| 11 | floor (ORCHESTRATOR) | `5199+` run, **0 FAILED** |
| 12 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 45-90 min. The split is a few lines; the goldens that move are the work.

## Trap-doors named in advance

- **Row 3 is the whole stone.** Rendering prettily and decoding wrongly is worse than the tagged
  carrier, which is at least honest. `try_ns` accepting the shape is not the same as the decoder
  reconstructing the original `::` spelling.
- **Row 5 is what a hasty fix deletes.** `verbatim_keyword`'s doc records exactly what it replaced —
  a bare String, *"a SILENT TYPE CHANGE … with no diagnostic."* Shrink what reaches it; keep it.
- **Row 8 is how an unrelated regression gets caught.** This moves EDN bytes corpus-wide. A diff you
  cannot attribute to `X/y` is the finding, not the noise.
- **Row 6 needs both directions** — the slash-in-name shape changes, everything else does not.
- ⚠ **Use the ONE door.** `receiver`/`method` are on `crates/wat-reader/src/identifier.rs`;
  `tests/lint/one_name_grammar.rs` makes hand-rolling a second split a RED. This arc has already been
  caught by that lint once.
