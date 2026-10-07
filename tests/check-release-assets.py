#!/usr/bin/env python3
"""Reject a packaged binary that does not embed the current Vite entry assets."""
from pathlib import Path
import argparse
import re

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('binary', nargs='?', type=Path, default=root / 'src-tauri/target/release/cyoa-manager')
args = parser.parse_args()
assets = re.findall(r'(?:src|href)="/?(assets/[^"]+)"', (root / 'dist/index.html').read_text())
if not assets:
    raise SystemExit('No frontend assets found; build the frontend first.')
contents = args.binary.read_bytes()
missing = [asset for asset in assets if asset.encode() not in contents]
if missing:
    raise SystemExit('Missing embedded frontend assets (development or stale binary): ' + ', '.join(missing))
print('Verified embedded frontend assets: ' + ', '.join(assets))
