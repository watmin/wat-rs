# NOTE — 12 more angle-strips, and my census was scoped from my own list

**Filed 2026-08-23**, immediately after `STONE-reap-the-angle-machinery` shipped (`131c7c299`).

## What happened

I instrumented four functions, measured 16.2M calls and zero type-heads, and briefed their deletion.
The rider deleted them, and then reported a population I had never censused: **~13 more hand-rolled
`.find('<')` strips** on declaration names and aggregate type keywords.

Measured myself rather than relayed — 13 sites, of which **one is ③'s wall and stays**:

```
src/runtime.rs:3127 · 3285 · 7492 · 18744 · 18894 · 18971        6
src/check.rs:12756 · 12895 · 13016                                3
src/types.rs:3156 · 3218                                          2
crates/wat-source-derive/src/lib.rs:73                            1
                                                                 ──
                                                                 12 candidates
src/types.rs:4688   ← ③'s WALL ("angle-bracket type parameters are illegal") — KEEPS
```

## ★ The error, and it is the third time in this arc

**I enumerated the four functions I had already named, instead of the RULE.**

The rule is *"code that parses a `<…>` suffix out of a name."* `canonical_callable_name`,
`split_type_params`, `split_name_and_type_params` and `split_method_name_type_params` are four
*instances* of it — the four that happened to be on my mind because earlier stones had named them. My
instrumentation then measured exactly those four, produced a beautifully precise number, and the
number was scoped to a list rather than to the property.

`[[feedback_scope_the_check_from_the_rule_not_the_diff]]`. Third instance this arc, and this one is
worse than the earlier two because **the instrument was excellent**: 16.2M calls, zero findings, a
per-function breakdown. A precise measurement of the wrong population is more convincing than a vague
one, not less.

The honest instrument was available and I did not use it: a grep for the BEHAVIOUR
(`find('<') | rfind('<') | contains('<') | ends_with('>')`) across `src/` and `crates/`, which is the
one command that produced the 13 above. I had run exactly that grep earlier in the campaign — it is
where the "48 angle sites" figure came from — and then narrowed to four when it came time to measure.

## What is NOT yet known about the 12

They are **almost certainly** the same dead-work class — angle suffixes cannot appear in any name any
more, so a `find('<')` on one cannot succeed. But *almost certainly* is what the last census said too.

Owed before any deletion:
- **Instrument all 12 the way the four were** — call counts and type-heads-found, over a full floor.
- The rider traced `construct_aggregate`'s suspicious comment (*"`~fqdn` may carry the full keyword"*)
  back to `wat/Record.wat`'s macro template and found it only ever splices a bare name. One down,
  eleven to go, and that tracing is the shape the rest need.
- ⚠ **`crates/wat-source-derive/src/lib.rs:73` cannot be treated as a sibling.** That crate structurally
  cannot depend on `wat` (it would cycle `wat-macros → wat-doc → wat-source-derive`), which is why it
  already carries an earned `rune:lint(one-param-spec)` exemption. It needs its own reasoning.

## Why the rune does not already cover them

`tests/lint/no_angle_suffix_strip.rs` bans the **balanced-suffix fingerprint** (`ends_with('>')`) —
the shape unique to the four deleted functions. It deliberately does NOT ban a bare `.find('<')`,
because doing so would have failed the baseline on these 12. That was the correct call for that stone
(a rune that cannot pass on a green tree is not a rune), and it is exactly why this NOTE exists: the
gap is known, bounded, and written down rather than left for a grep to rediscover.

Kin: `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`,
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`, and the campaign's own recurring
lesson — *a search for a character that no longer exists does not fail, it succeeds wrongly.*
