#!/usr/bin/env python3
"""Source/export/privacy, documentation links and Linux-only configuration gate."""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('source_export', ROOT / 'scripts/prepare-github.py')
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)
files = exporter.source_files()
for name, data, _ in files:
    if name.endswith('.py'):
        ast.parse(data.decode(), filename=name)
    if name.endswith('.md'):
        text = re.sub(r'```[\s\S]*?```', '', data.decode())
        for match in re.finditer(r'\[[^\]]*\]\(([^)]+)\)', text):
            target = match[1].strip('<>').split('#')[0]
            if not target or re.match(r'[a-zA-Z]+:', target):
                continue
            path = (ROOT / name).parent / target
            assert path.exists(), f'{name}: broken local link {target}'
package = json.loads((ROOT / 'package.json').read_text())
tauri = json.loads((ROOT / 'src-tauri/tauri.conf.json').read_text())
cargo = tomllib.loads((ROOT / 'src-tauri/Cargo.toml').read_text())
assert package['version'] == tauri['version'] == cargo['package']['version']
assert set(tauri['bundle']['targets']) <= {'appimage', 'deb', 'rpm'}
assert tauri['identifier'] == 'com.om1cr0n.cyoa-manager'
assert not tauri['app']['windows'][0].get('maximized', False)
assert not tauri['app']['windows'][0].get('fullscreen', False)
workflow = (ROOT / '.github/workflows/linux.yml').read_text()
assert 'ubuntu-24.04' in workflow and not re.search(r'windows-latest|macos-latest|release-multi-platform', workflow)
for script in [ROOT / 'scripts/launch-linux.sh']:
    subprocess.run(['bash', '-n', str(script)], check=True)
with tempfile.TemporaryDirectory(prefix='cyoa-source-check-') as temp:
    temp = Path(temp)
    archive = temp / 'source.tar.gz'
    exporter.export(archive, files)
    report = json.loads(archive.with_name(archive.name + '.manifest.json').read_text())
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == report['sha256']
    with tarfile.open(archive) as tar:
        names = tar.getnames()
        assert all(not Path(name).is_absolute() and '..' not in Path(name).parts for name in names)
        assert not any(set(Path(name).parts) & exporter.SKIP for name in names)
    fixture = temp / 'fixture'
    (fixture / 'src').mkdir(parents=True)
    (fixture / 'package.json').write_text('{}')
    private = fixture / 'src/.env'
    private.write_text('private configuration')
    try:
        exporter.source_files(fixture)
        raise AssertionError('Private file was accepted')
    except ValueError:
        pass
    private.unlink()
    link = fixture / 'src/link'
    link.symlink_to(temp / 'outside')
    try:
        exporter.source_files(fixture)
        raise AssertionError('Symlink was accepted')
    except ValueError:
        pass
print(f'Source/export readback, privacy exclusions, Python/shell syntax, documentation links, versions and Linux-only configuration passed ({len(files)} files).')
