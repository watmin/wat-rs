#!/usr/bin/env python3
"""PROBE 296-L — the bare-`is_err()` census. THE INSTRUMENT THAT PRODUCED THE NUMBER.

Committed because a count whose instrument lives in a session tmp dir is a number nobody can
reproduce. `[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

WHY NOT GREP: an `assert!` spans lines, so a line-based grep cannot see the statement. A first
line-based attempt reported **70**; this statement-scoped, paren-balanced instrument reports
**150** — under by more than half, and that undercount read as solid.

WHAT IT CLASSIFIES: every paren-balanced `assert!( ... )` whose body mentions `.is_err()`. BARE =
the body checks nothing about WHICH error (no `matches!`, no `assert_check_error_present`, no
`.kind`, no `unwrap_err`, no `err_kind`, no `StartupError::`). KINDED = it does.

VALIDATED against five ground-truth controls before its number was quoted (single-line bare ->
BARE; wrapped multi-line bare -> BARE; `matches!`-guarded -> KINDED; `is_ok()` -> neither;
`if x.is_err()` control-flow -> neither). All five behaved as specified.

Run from the repo root:  python3 docs/arc/.../PROBE-296-L-bare-is-err-census.py

SUPERSEDED BY PHASE 3: `tests/lint/no_bare_is_err.rs` is the permanent Rust wall. This probe is the
pre-work measurement and the second opinion, not the ratchet.
"""
import os,re,sys
def stmts(src):
    # yield (line_no, text) for each assert!( ... ) statement, paren-balanced
    out=[]
    for m in re.finditer(r'\bassert!\s*\(', src):
        i=m.end()-1; d=0
        while i<len(src):
            c=src[i]
            # A char literal holding a paren -- e.g. `!w.starts_with('(')` -- desyncs the
            # counter and swallows the rest of the file into one "statement". Found by the
            # Phase 2 tail rider, in this instrument, after it had produced the 150.
            # `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
            if c=="'" and i+2 < len(src) and src[i+2]=="'":
                i+=3; continue
            if c=='"':
                i+=1
                while i<len(src) and src[i]!='"':
                    i += 2 if src[i]=='\\' else 1
                i+=1; continue
            if c=='(': d+=1
            elif c==')':
                d-=1
                if d==0: break
            i+=1
        out.append((src[:m.start()].count('\n')+1, src[m.start():i+1]))
    return out
KIND=('matches!','assert_check_error_present','.kind','unwrap_err','err_kind','StartupError::')
bare=[]; kinded=[]
for root,dirs,files in os.walk('tests'):
    dirs[:]=[d for d in dirs if d not in ('target','.claude')]
    for f in files:
        if not f.endswith('.rs'): continue
        p=os.path.join(root,f); src=open(p,encoding='utf-8',errors='replace').read()
        for ln,t in stmts(src):
            if '.is_err()' not in t: continue
            (kinded if any(k in t for k in KIND) else bare).append((p,ln,' '.join(t.split())[:90]))
print("BARE  is_err assertions (no kind check):",len(bare))
print("KINDED is_err assertions:",len(kinded))
print("\n-- POSITIVE CONTROL: the Stone F fixture's test must appear --")
print([x for x in bare if 'guard_ensure' in x[0] and x[1] in range(150,160)] or "  ⛔ NOT FOUND")
print("\n-- NEGATIVE CONTROL: a kinded site must NOT be in bare --")
print(kinded[0] if kinded else "  (none kinded)")
print("\n-- top files --")
from collections import Counter
for f,c in Counter(p for p,_,_ in bare).most_common(10): print(f"  {c:3}  {f}")
