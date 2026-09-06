# EXPECTATIONS — STONE 251.8b: the tuple is stored

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **the tuple is STORED** | `Identifier` holds the namespace and the name as separate fields. **`namespace()` returns a field, not a `rfind` result** |
| 3 | ★★★ **and every accessor keeps its `&str` signature** | `as_str` · `namespace` · `leaf` · `path` · `receiver` · `method` · `deprimed` — all still `-> &str`. **251.8a promised "behind this same signature"** |
| 4 | ★★★ **`foo` is `[$bound, foo]`** | `Identifier::bare("foo").namespace()` = `$bound`, `is_reference()` = `false`. The builder's model, held rather than derived |
| 5 | ★★★ **`wat.core/+` is `[wat.core, +]`** | `namespace()` = `wat.core`, and the name is `+` |
| 6 | ★★★ **NO accessor re-derives** | `grep -c "rfind('/')" identifier.rs` over the accessor bodies = **0**. **Storing the tuple and then still splitting on read is the defect with extra fields** |
| 7 | ★★★ **construction derives ONCE, at the chokepoint** | the split happens in `Identifier::bare` and nowhere else. 147 call sites reach it; none of them split |
| 8 | ★★ `scopes` is untouched and NOT in the tuple | `add_scope`/`scopes` unchanged; hygiene rides alongside. **A three-member name is the illegal shape** |
| 9 | ★★★ **behaviour is IDENTICAL for every existing input** | every accessor returns today's answer for today's inputs. **This is a representation change; a behaviour change hiding in it is the risk** |
| 10 | ★★★ **including the shapes that are currently WRONG** | `wat.core//` still answers whatever it answers today. ⚠ **If the reader's split disagrees with the builder's model, REPORT it — do not fix it here.** Row 9 forbids improving behaviour in a representation stone |
| 11 | ★★ nothing outside the crate changes | `git diff` outside `crates/wat-reader/` is empty, or every file named and justified |
| 12 | ★★ negative control | revert one accessor to a `rfind` derivation → a test asserting the stored field goes RED. Committed |
| 13 | floor (ORCHESTRATOR) | `5199+` run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 50-90 min. The representation is small and fully encapsulated; row 9's
"identical for every input" is the work.

## Trap-doors named in advance

- **Row 9 is the whole stone.** This buys nothing today — no bug is fixed, no output changes. Its
  value is entirely that it makes the NEXT four things possible. **A behaviour change smuggled in is
  pure cost with no visible cause.**
- **Row 10 is where that pressure will bite.** `wat.core//` reads as `["wat.core/", ""]` today,
  which contradicts the builder's `[wat.core, /]`. It is a **reader** question and fixing it here
  makes row 9 unprovable. **Report it; it is the next stone, not this one.**
- **Row 6 is the point of storing.** Fields that are recomputed on read are a bigger struct doing
  the same guessing.
- **Row 3 is a promise 251.8a made.** If a stored tuple genuinely cannot keep `as_str() -> &str`,
  **STOP and report** — the alternative changes a signature across 147 sites and is the builder's
  call, not the strike's.
- ⚠ **`scopes` is macro hygiene, not a name part.** Folding it in makes a three-member name.
