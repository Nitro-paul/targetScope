#!/usr/bin/env python3
# doc/development.md#checks
"""Checks the documentation and the comments of the repository.

1. Every relative link and anchor in the Markdown files exists.
2. Every `doc/<file>.md#<anchor>` reference in code and data files exists.
3. Comments in code only contain such references or test ids (and nothing else).

Usage: tools/check_docs.py [<repo folder>]; exit code 1 if anything is wrong."""
import os
import re
import subprocess
import sys

root = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), '..'))
tracked = subprocess.run(['git', 'ls-files'], cwd=root, capture_output=True, text=True, check=True).stdout.split()
problems = []

HEADING = re.compile(r'^#{1,6}\s+(.*?)\s*#*$')


def slug(heading):
    heading = re.sub(r'[`*]', '', heading.strip().lower())
    return re.sub(r'[^\w\- ]', '', heading).replace(' ', '-')


_anchors = {}


def anchors(path):
    if path not in _anchors:
        found, seen, in_code = set(), {}, False
        for line in open(os.path.join(root, path), encoding='utf-8'):
            if line.startswith('```'):
                in_code = not in_code
            elif not in_code and (m := HEADING.match(line)):
                s = slug(m.group(1))
                n = seen.get(s, 0)
                seen[s] = n + 1
                found.add(s if n == 0 else f'{s}-{n}')
        _anchors[path] = found
    return _anchors[path]


def read(path):
    try:
        return open(os.path.join(root, path), encoding='utf-8').read()
    except (UnicodeDecodeError, OSError):
        return None


# 1. Markdown links
for path in (p for p in tracked if p.endswith('.md')):
    in_code = False
    for number, line in enumerate(read(path).split('\n'), 1):
        if line.startswith('```'):
            in_code = not in_code
        if in_code:
            continue
        for m in re.finditer(r'\[[^\]]*\]\(([^)\s]+)\)', line):
            target = m.group(1)
            if re.match(r'^(https?:|mailto:)', target):
                continue
            file_part, _, fragment = target.partition('#')
            resolved = path if not file_part else os.path.normpath(os.path.join(os.path.dirname(path), file_part))
            if not os.path.exists(os.path.join(root, resolved)):
                problems.append(f'{path}:{number}: file not found: {target}')
            elif fragment and resolved.endswith('.md') and fragment not in anchors(resolved):
                problems.append(f'{path}:{number}: anchor not found: {target}')

# 2. doc references in code and data
DOC_REF = re.compile(r'(?<![\w/.-])((?:\.\./)*doc/[\w./-]+\.md)(?:#([\w.-]+))?')
for path in tracked:
    text = None if path.endswith(('.md', '.lock', '.pdf', '.png', '.jpg', '.ttf')) else read(path)
    if text is None:
        continue
    for number, line in enumerate(text.split('\n'), 1):
        for m in DOC_REF.finditer(line):
            target = re.sub(r'^(\.\./)+', '', m.group(1))
            if not os.path.exists(os.path.join(root, target)):
                problems.append(f'{path}:{number}: file not found: {target}')
            elif m.group(2) and m.group(2) not in anchors(target):
                problems.append(f'{path}:{number}: anchor not found: {target}#{m.group(2)}')

# 3. comment rule
CODE = ('.rs', '.cpp', '.hpp', '.h', '.js', '.css', '.sh', '.toml', '.cmake', 'CMakeLists.txt', '.py')
REF = r'doc/[\w./*-]+\.md(#[\w.-]+)?'
TEST_ID = r'[A-Z]{2,4}(-[A-Z0-9]+)+'
ALLOWED = re.compile(rf'^(({REF})|({TEST_ID}))(, (({REF})|({TEST_ID})))*$')
for path in tracked:
    if not path.endswith(CODE) or path.startswith('testdata/'):
        continue
    text = read(path)
    if text is None:
        continue
    hash_comments = path.endswith(('.sh', '.toml', '.cmake', 'CMakeLists.txt', '.py'))
    for number, line in enumerate(text.split('\n'), 1):
        s = line.strip()
        if number == 1 and s.startswith('#!'):
            continue
        if s.startswith('//') or (hash_comments and s.startswith('#') and not s.startswith('#[')):
            body = re.sub(r'^(//[/!]?|#)\s?', '', s)
            if body and not ALLOWED.match(body):
                problems.append(f'{path}:{number}: comment is not a documentation reference: {s[:70]}')
        elif path.endswith(('.rs', '.cpp', '.hpp', '.h', '.js')) and (m := re.search(r'(?<=\S)\s+//\s?(?!doc/)(.*)$', line)):
            if line[:m.start()].count('"') % 2 == 0 and '://' not in line:
                problems.append(f'{path}:{number}: trailing comment: {m.group(0).strip()[:60]}')

# 3b. block comments in C and C++ files
for path in tracked:
    if not path.endswith(('.h', '.hpp', '.cpp', '.css')):
        continue
    for number, line in enumerate(read(path).split('\n'), 1):
        for m in re.finditer(r'/\*(.*?)(\*/|$)', line):
            body = m.group(1).strip()
            if not m.group(2) or (body and not ALLOWED.match(body)):
                problems.append(f'{path}:{number}: block comment is not a documentation reference: {line.strip()[:60]}')

for p in problems:
    print(p)
print(f'{len(problems)} problem(s)', file=sys.stderr)
sys.exit(1 if problems else 0)
