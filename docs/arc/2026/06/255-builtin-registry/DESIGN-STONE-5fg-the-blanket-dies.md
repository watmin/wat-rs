# DESIGN — STONE ⑤-F+G: THE BLANKET DIES

Arc 255's founding sentence, shipped. Everything ahead of it has landed; this is the deletion and
the seven expectations that move with it.

```rust
-  if is_reserved_prefix(head) { return true; }
+  if crate::intrinsic::registry().contains(head) { return true; }
```

plus the `:rust::*` rung that routes to the authority which already validates it.

## ★★★ THE CORPUS IS ALREADY CLEAN — measured on this tree, with the deletion applied

```
843 corpus files          36 refuse   ← EXACTLY the clean-main baseline
reserved-prefix names still unresolved ......... 0
TRUE dependents (pass today, break then) ....... 0
```

Stones ①–⑤E did their job. **Nothing in `wat/`, `wat-scripts/` or `wat-tests/` depends on the
blanket any more.** The floor waterfall went `133 → 111 → 97 → 7`.

## The seven, and every one is an EXPECTATION, not a defect

### ⛔ TWO UNIT TESTS WHOSE SUBJECT IS THE BLANKET ITSELF

`src/resolve/mod.rs:177` — and its comment is the blanket's own specification:

> ```rust
> // These aren't implemented yet but shouldn't fail resolution —
> // they're under reserved prefixes that the spec carves out.
> ```

**A test asserting that UNIMPLEMENTED NAMES MUST RESOLVE.** That is precisely the property arc 255
exists to end, written down as a guarantee. Measured, of its four names:

```
:wat::kernel::send        registered intrinsic          ✓ resolves
:wat::config::dim-count   registered intrinsic + scheme ✓ resolves
:wat::holon::Subtract     a STDLIB MACRO (wat/holon/Subtract.wat) — resolves via `macros`
                          at full load; wat-tests/holon/Subtract.wat --checks clean WITH the
                          deletion. It fails only in this test's BARE environment, where no
                          stdlib is loaded and the blanket was covering that.
:wat::config::set-dim-count!   ⛔ REAL AND UNREGISTERED — a live arm at src/config.rs:441,
                          no row, no scheme.
```

**Disposition:** rewrite both tests to assert what is now TRUE — a reserved-prefix name resolves
**iff the registry knows it** — which is the arc's founding sentence as a unit test. And declare
`set-dim-count!`, the one real verb among them.

### ⛔ TWO REMEDY TESTS THAT PINNED THE BLANKET'S SILENCE

`probe_arc241_stone10_remedy` c04 / c08. c04 feeds `:wat::core::xyzzy` — a deliberately distant
bogus name — and expects:

```
"<startup succeeded — no error to display>"
```

**The blanket made a nonsense name type-check clean, and the test froze that as the expected
string.** With the deletion it is properly refused, with NO "did you mean" — which is still exactly
what the test's own claim says ("distant-unknown should NOT produce 'did you mean'"). Only the
literal expectation moves; the assertion's meaning is unchanged and better served.

### ⛔ ONE NEGATIVE FIXTURE WHOSE ERROR MOVES UPSTREAM — family F, as designed

`probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time`. `ProcessLaunch/bogus-field`
was caught by the CHECKER; now `resolve` catches it first. Refused either way, one pass earlier.

### ★★ TWO RATCHETS FIRING ON CUE — this is their whole purpose

`probe_arc255_the_blanket_hides_a_phantom_head` — both rows, planted in stone ④ pinning
PRE-deletion behaviour, verified non-vacuous then by applying this very deletion:

```
bogus_rete_head              --check exit 0  →  must become exit 1
dot_spelling_reaches...      --check exit 0  →  must become exit 1
```

⛔ **Updating them is not maintenance — it is this stone's PROOF.** Their headers say a red here
means the blanket died, and the new goldens are the evidence. `:wat::rete::f64::>X`, a bogus head
that has type-checked clean for as long as it has existed, finally fails. That is arc 255's
founding promise — *"the undefined-func class dies as a SIDE EFFECT of fixing the real defect"* —
collecting its last scalp.

## What must NOT happen

⛔ **Do not weaken a remedy assertion to make it pass.** c04's claim is "no *did you mean* for a
distant unknown." If the new error carries a remedy, that is a FINDING about the remedy threshold,
not something to assert away.

⛔ **Do not re-add a prefix acceptance for anything.** `:rust::*` defers to `UseDeclarations::covers`,
which validates on the very next lines — that is routing, not a blanket. Nothing else gets one.

⛔ **Do not touch the reserved-prefix WALL.** `resolve/registration.rs:129` refuses userland
definitions under `:wat::*` / `:rust::*` and is a DIFFERENT consumer of the same predicate. Seven
committed rows in `probe_arc255_the_reserved_prefix_wall_is_not_the_blanket` hold that line, and
they were measured green both with the blanket AND with it deleted.

## Out of scope = REJECTED

- **The dot flip.** Next, and now unblocked — the seam's ordering caveat is satisfied.
- **`:wat::config::set-*!`'s siblings** beyond what the two tests name. Declare what the floor
  demands; a sweep of the config family is its own stone.
