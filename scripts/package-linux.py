#!/usr/bin/env python3
"""Package a Linux release with its viewer engines; no library or private test data."""
import argparse
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=root / 'src-tauri/target/release/cyoa-manager')
args = parser.parse_args()
if sys.platform != 'linux':
    raise SystemExit('This fork packages Linux builds only.')
subprocess.run([sys.executable, str(root / 'tests/check-release-assets.py'), str(args.binary)], check=True)
name = 'cyoa-manager-linux-' + platform.machine()
output = root / 'release' / name
if output.exists():
    raise SystemExit(f'{output} already exists; move it aside before packaging another build.')
try:
    (output / 'runtime/bin').mkdir(parents=True)
    shutil.copy2(args.binary, output / 'runtime/bin/cyoa-manager')
    shutil.copytree(root / 'public/viewers', output / 'runtime/viewers')
    (output / 'runtime/icons').mkdir()
    for source, target in [('128x128.png','cyoa-manager-128.png'),('32x32.png','cyoa-manager-32.png')]:
        shutil.copy2(root / 'src-tauri/icons' / source, output / 'runtime/icons' / target)
    shutil.copy2(root / 'scripts/launch-linux.sh', output / 'launch.sh')
    (output / 'launch.sh').chmod(0o755)
    shutil.copy2(root / 'scripts/install-linux-launcher.py', output / 'install-menu.py')
    for file in ['README.md','LICENSE']:
        shutil.copy2(root / file, output / file)
    shutil.copytree(root / 'docs', output / 'docs')
    (output / 'tests').mkdir()
    shutil.copy2(root / 'tests/README.md', output / 'tests/README.md')
    archive = output.with_suffix('.tar.gz')
    pending = archive.with_suffix('.gz.pending')
    with tarfile.open(pending, 'w:gz') as tar:
        tar.add(output, arcname=name)
    pending.replace(archive)
except Exception:
    shutil.rmtree(output)
    raise
print(archive)
