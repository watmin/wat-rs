# EXPECTATIONS — the sketch and the wall

Written BEFORE the strike so the result cannot move the goalposts. Every row's bar is derived from
the RULING's rule or from a mechanism in the source, not from what I expect to see.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | no `HolonAST` left in the sketch path | `grep -nE 'HolonAST' src/special_forms.rs src/reflect/lookup.rs src/reflect/verbs.rs` | **zero** code lines; comments recording the retirement may remain (FM 14 bucket C) |
| 2 | the emitted signature is unchanged | render one signature before and after | byte-identical strings |
| 3 | `require_bundle` lives with its callers | `grep -rn 'fn require_bundle' src/` | `src/intrinsic/holon/atom.rs`, and nothing in `runtime.rs` but a moved-to marker |
| 4 | the wall exists and is registered | `cargo nextest run --release -E 'test(holon_is_vsa_only)'` | 1 test, passing |
| 5 | the wall is NOT vacuous | inject `HolonAST::bundle(vec![])` into a non-home file, run row 4 | **RED**, naming that file and line |
| 6 | the wall arms at ZERO | row 4 with the injection removed | green with no runes beyond the one at `runtime.rs:~20125` |
| 7 | nothing else moved | `git diff --stat` | only the files parts 1–4 name |
| 8 | the floor holds | orchestrator, centrally, once | 5129/5129 or better, 0 failed |
| 9 | clippy holds | `cargo clippy --release --all-targets -- -D warnings` | 0 |

Row 2's derivation, so the bar is not my guess: `holon_to_watast` maps `Bundle→List`,
`keyword(head)` back through `format!(":{}", s)`, and `symbol(s)` to
`Symbol(Identifier::bare(s))` — the three arms `sketch()` uses compose to the identity on
`WatAST::List/Keyword/Symbol`. If row 2 shows a difference, the composition claim is false and
that is the finding, not a nuisance to patch.

Row 6 is the load-bearing one. `tests/lint/no_rc_use.rs`'s own doctrine: *"a lint raised at zero is
a wall, a lint raised at 1306 is a campaign."* A wall that ships with a pile of exemptions is a
campaign wearing a wall's clothes.

## Independent prediction

**Runtime 25–40 minutes.** Parts 1–3 are ~20 lines of mechanical edits across five files. Part 4 is
the real work: the construction-vs-pattern discrimination is genuinely fiddly, and the sabotage
round-trip costs two builds.

## Trap doors — named before, so they are not surprises after

1. **The slot-name precondition (STOP-1).** If any slot is `"nil"` or `:`-prefixed, the identity
   claim is false. I read the current slots and believe none is — but a failure to find is not a
   proof of absence, which is exactly why the rider must ENUMERATE rather than confirm my reading.
2. **The construction-vs-pattern discrimination.** A regex cannot parse Rust. Whatever rule the
   rider picks will have a blind spot; the honest outcome is that the blind spot is NAMED in the
   module doc, not that it does not exist. A wall that claims to see everything is the failure.
3. **`Binding::SpecialForm` may be near-unreachable.** Its own doc says the variant is reachable
   today only for the three names the registry does not carry. If a test therefore never exercises
   the changed field, row 2 is being satisfied through a path the tests do not cover — say so
   plainly rather than reporting a green.
4. **The move might not be pure.** `require_bundle` is `pub(crate)` in `runtime.rs`; its new home
   is a module, and visibility or import churn could ride along. That is STOP-3, and a small
   `use` adjustment is not a behaviour change — a signature or error-kind change is.

## What I will do on return

Re-run rows 1–7 myself before scoring; rows 8 and 9 are mine alone and are the only verdict on
green. The rider's numbers are a hypothesis until a current `file:line` confirms them.
