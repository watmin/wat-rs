#!/usr/bin/env python3
"""
Transform legacy let binding forms to flat Vector form.

Legacy: (:wat::core::let ((name expr) (name expr)) body)
New:    (:wat::core::let [name expr name expr] body)

Three binder shapes:
- Bare symbol: (name expr) -> name expr
- Typed legacy: ((name :T) expr) -> name expr (type dropped)
- Destructure: ((a b c) expr) -> [a b c] expr
"""
import sys
import os


def parse_one_form(text, start):
    """
    Parse one s-expression form starting at text[start].
    Returns (end_index, form_text).
    text[start] should be the opening character.
    """
    i = start
    # skip whitespace
    while i < len(text) and text[i] in ' \t\n\r':
        i += 1
    if i >= len(text):
        return i, ''

    ch = text[i]
    if ch == '(':
        depth = 0
        start_pos = i
        while i < len(text):
            c = text[i]
            if c == '"':
                i += 1
                while i < len(text) and text[i] != '"':
                    if text[i] == '\\':
                        i += 1
                    i += 1
                i += 1  # closing "
                continue
            if c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
                if depth == 0:
                    return i + 1, text[start_pos:i+1]
            i += 1
        return i, text[start_pos:i]
    elif ch == '[':
        depth = 0
        start_pos = i
        while i < len(text):
            c = text[i]
            if c == '"':
                i += 1
                while i < len(text) and text[i] != '"':
                    if text[i] == '\\':
                        i += 1
                    i += 1
                i += 1
                continue
            if c == '[':
                depth += 1
            elif c == ']':
                depth -= 1
                if depth == 0:
                    return i + 1, text[start_pos:i+1]
            i += 1
        return i, text[start_pos:i]
    elif ch == '"':
        start_pos = i
        i += 1
        while i < len(text) and text[i] != '"':
            if text[i] == '\\':
                i += 1
            i += 1
        i += 1  # closing "
        return i, text[start_pos:i]
    elif ch == '`':
        # backtick quasiquote - treat as single char token or scan ahead
        start_pos = i
        i += 1
        # The next token is the form
        end, _ = parse_one_form(text, i)
        return end, text[start_pos:end]
    elif ch == "'":
        start_pos = i
        i += 1
        end, _ = parse_one_form(text, i)
        return end, text[start_pos:end]
    else:
        # symbol/keyword/number
        start_pos = i
        while i < len(text) and text[i] not in ' \t\n\r()[]"':
            i += 1
        return i, text[start_pos:i]


def parse_binding_list(binding_list):
    """
    Parse a binding list: ((binder expr) (binder expr) ...)
    binding_list is the text including outer parens.
    Returns list of (binder_text, expr_text) tuples.
    """
    # strip outer parens
    inner = binding_list[1:-1]

    pairs = []
    i = 0
    while i < len(inner):
        # skip whitespace
        while i < len(inner) and inner[i] in ' \t\n\r':
            i += 1
        if i >= len(inner):
            break
        if inner[i] != '(':
            # unexpected - not a pair
            break

        # parse one pair
        pair_end, pair = parse_one_form(inner, i)
        i = pair_end

        if not pair:
            break

        # pair is (binder expr)
        # parse binder (first element inside pair)
        pair_inner = pair[1:-1]  # strip outer parens of pair

        # skip whitespace
        j = 0
        while j < len(pair_inner) and pair_inner[j] in ' \t\n\r':
            j += 1

        binder_end, binder = parse_one_form(pair_inner, j)
        j = binder_end

        # skip whitespace
        while j < len(pair_inner) and pair_inner[j] in ' \t\n\r':
            j += 1

        # rest is expr (may have trailing whitespace)
        expr = pair_inner[j:].rstrip()

        pairs.append((binder, expr))

    return pairs


def transform_binder(binder):
    """
    Transform a binder.
    - Symbol -> same symbol
    - (name :Type) -> name  (typed legacy - drop type)
    - (a b c) -> [a b c]  (destructure)
    """
    binder = binder.strip()
    if not binder.startswith('('):
        # bare symbol
        return binder

    # It's a list binder
    inner = binder[1:-1].strip()
    parts = inner.split()

    if len(parts) >= 2 and parts[1].startswith(':'):
        # Typed legacy: (name :Type) -> name
        return parts[0]
    else:
        # Destructure: (a b c) -> [a b c]
        return '[' + inner + ']'


def get_leading_ws(text, start):
    """Get whitespace between start and first non-whitespace."""
    j = start
    ws = ''
    while j < len(text) and text[j] in ' \t\n\r':
        ws += text[j]
        j += 1
    return ws, j


def get_indent_of_pairs(binding_list_inner):
    """
    Get the indentation of the first pair inside the binding list inner text.
    Returns the indent string (spaces/tabs after the last newline before first '(').
    """
    i = 0
    indent = ''
    while i < len(binding_list_inner):
        ch = binding_list_inner[i]
        if ch == '\n':
            indent = ''
        elif ch in ' \t':
            indent += ch
        elif ch == '(':
            return indent
        i += 1
    return indent


def format_new_bindings(pairs, binding_list_inner):
    """
    Format new binding vector content.
    pairs: list of (binder, expr) tuples (already transformed)
    binding_list_inner: original inner text (for whitespace guidance)
    Returns the content to go inside [...]
    """
    if not pairs:
        return ''

    # Get the whitespace/indent of first pair
    first_ws, _ = get_leading_ws(binding_list_inner, 0)
    pair_indent = get_indent_of_pairs(binding_list_inner)

    parts = []
    for binder, expr in pairs:
        new_binder = transform_binder(binder)
        parts.append(new_binder + ' ' + expr)

    if len(parts) == 1:
        # Check if original was multiline
        if '\n' in binding_list_inner:
            return first_ws + parts[0]
        else:
            return parts[0]
    else:
        # Multiline: use newline + indent between pairs
        separator = '\n' + pair_indent
        return first_ws + separator.join(parts)


def transform_let_bindings(content):
    """
    Transform all legacy let binding forms in content.
    Returns (new_content, change_count).
    """
    LET_PAT = '(:wat::core::let'
    result = []
    i = 0
    change_count = 0

    while i < len(content):
        idx = content.find(LET_PAT, i)
        if idx == -1:
            result.append(content[i:])
            break

        # copy up to the let
        result.append(content[i:idx])
        i = idx + len(LET_PAT)

        # check what follows after whitespace
        ws, j = get_leading_ws(content, i)

        if j < len(content) and content[j] == '(':
            # Legacy binding list! Transform it.
            # Parse the binding list
            binding_end, binding_list = parse_one_form(content, j)

            if not binding_list:
                # couldn't parse
                result.append(LET_PAT)
                continue

            # Parse pairs from binding list
            binding_inner = binding_list[1:-1]
            pairs = parse_binding_list(binding_list)

            if not pairs:
                # Empty or couldn't parse pairs - keep as-is
                result.append(LET_PAT)
                result.append(ws)
                # DON'T change it, just keep original
                result.append(binding_list)
                i = binding_end
                continue

            # Format new binding
            new_content = format_new_bindings(pairs, binding_inner)
            new_binding = '[' + new_content + ']'

            result.append(LET_PAT)
            result.append(ws)
            result.append(new_binding)
            i = binding_end
            change_count += 1
        else:
            # Already new format or empty bindings
            result.append(LET_PAT)
            # i stays at j (after whitespace we didn't consume)
            # Actually we need to output the whitespace too
            result.append(ws)
            i = j

    return ''.join(result), change_count


def process_file(filepath):
    """Process a single file. Returns True if changed."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"ERROR reading {filepath}: {e}", file=sys.stderr)
        return False

    new_content, changes = transform_let_bindings(content)

    if changes > 0:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"  {filepath}: {changes} site(s) changed")
        return True
    return False


def main():
    if len(sys.argv) < 2:
        print("Usage: transform_let.py <file1> [file2] ...", file=sys.stderr)
        sys.exit(1)

    total_changed = 0
    total_files = 0
    for path in sys.argv[1:]:
        if os.path.isfile(path):
            if process_file(path):
                total_files += 1
                total_changed += 1
        else:
            print(f"SKIP (not a file): {path}", file=sys.stderr)

    print(f"Done: {total_files} file(s) changed")


if __name__ == '__main__':
    main()
