#!/usr/bin/env python3
"""Validate/export this source tree without attaching Git, fetching or uploading."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORIES = {'src', 'src-tauri', 'public', 'scripts', 'tests', 'docs', '.github', '.vscode'}
FILES = {'README.md', 'HANDOFF.md', 'LICENSE', '.gitignore', 'package.json', 'pnpm-lock.yaml',
         'pnpm-workspace.yaml', 'index.html', 'tsconfig.json', 'tsconfig.node.json', 'vite.config.ts'}
SKIP = {'.git', 'node_modules', 'target', 'dist', 'release', 'save', 'cyoas', 'graft', '__pycache__',
        'results', 'fixtures', 'validation', 'backups', 'gen'}
PATTERNS = [
    ('API key', re.compile(r'AIza[0-9A-Za-z_-]{30,}')),
    ('private key', re.compile(r'-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----')),
    ('GitHub token', re.compile(r'(?:ghp_|github_pat_)[A-Za-z0-9_]{20,}')),
    ('personal home path', re.compile(r'/' + 'home' + r'/[^\s/]+/|[A-Za-z]:\\Users\\[^\\\s]+\\')),
]

def source_files(root=ROOT):
    result = []
    candidates = [root / name for name in FILES if (root / name).exists()]
    for name in sorted(DIRECTORIES):
        top = root / name
        if top.is_symlink():
            raise ValueError(f'Refusing source symlink: {name}')
        if not top.is_dir():
            continue
        for folder, directories, names in os.walk(top, followlinks=False):
            directories[:] = [name for name in directories if name not in SKIP]
            for name in directories:
                if (Path(folder) / name).is_symlink():
                    raise ValueError('Refusing source directory symlink')
            candidates.extend(Path(folder) / name for name in names)
    for path in sorted(candidates):
        relative = path.relative_to(root)
        if any(part in SKIP for part in relative.parts):
            continue
        if relative.parts[0] not in DIRECTORIES and relative.as_posix() not in FILES:
            continue
        if path.is_symlink():
            raise ValueError(f'Refusing source symlink: {relative}')
        if not path.is_file():
            continue
        if path.name.startswith('.env') or path.suffix in {'.sqlite', '.sqlite3', '.db', '.pem', '.key', '.log', '.pending', '.pyc', '.bak', '.backup', '.orig', '.rej', '.tmp'}:
            raise ValueError(f'Private/generated file in source: {relative}')
        data = path.read_bytes()
        if b'\0' not in data:
            text = data.decode('utf-8', errors='replace')
            for reason, pattern in PATTERNS:
                if pattern.search(text):
                    raise ValueError(f'{reason} detected in {relative}; value omitted')
        result.append((relative.as_posix(), data, path.stat().st_mode & 0o777))
    if not any(name == 'package.json' for name, _, _ in result):
        raise ValueError('Source package.json is missing')
    return result

def export(destination, files):
    destination.parent.mkdir(parents=True, exist_ok=True)
    pending = destination.with_name(destination.name + '.pending')
    manifest = {name: hashlib.sha256(data).hexdigest() for name, data, _ in files}
    try:
        with tarfile.open(pending, 'w:gz') as archive:
            for name, data, mode in files:
                info = tarfile.TarInfo(name)
                info.size = len(data)
                info.mode = 0o755 if mode & 0o111 else 0o644
                archive.addfile(info, io.BytesIO(data))
        with tarfile.open(pending) as archive:
            found = {entry.name: hashlib.sha256(archive.extractfile(entry).read()).hexdigest()
                     for entry in archive if entry.isfile()}
        if found != manifest:
            raise ValueError('Source export readback failed')
        pending.replace(destination)
        report = destination.with_name(destination.name + '.manifest.json')
        temporary = report.with_name(report.name + '.pending')
        temporary.write_text(json.dumps({'archive': destination.name, 'sha256': hashlib.sha256(destination.read_bytes()).hexdigest(), 'files': manifest}, indent=2) + '\n')
        temporary.replace(report)
    finally:
        pending.unlink(missing_ok=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Validate only; create no archive')
    parser.add_argument('--output', type=Path, default=ROOT / 'release/cyoa-manager-source.tar.gz')
    args = parser.parse_args()
    files = source_files()
    print(f'Validated {len(files)} source files; no library, saves, profiles, backups, Git metadata or build output included.')
    if not args.check:
        export(args.output.resolve(), files)
        print(args.output.resolve())
    print('Git remains detached. Nothing fetched, committed or uploaded.')

if __name__ == '__main__':
    main()
