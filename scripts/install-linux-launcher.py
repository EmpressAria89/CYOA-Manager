#!/usr/bin/env python3
"""Install a separate menu entry; dry-run unless --apply. Preserve upstream entry."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--apply', action='store_true')
parser.add_argument('--directory', type=Path, help='Extracted Linux package folder')
args = parser.parse_args()
script = Path(__file__).resolve()
app = args.directory.resolve() if args.directory else (script.parent if (script.parent / 'launch.sh').exists() else script.parents[2])
launcher = app / 'launch.sh'
icon = app / 'runtime/icons/cyoa-manager-128.png'
# Desktop Exec has its own escaping rules; this is one quoted literal argument.
def desktop_arg(value):
    value = str(value).replace('%', '%%')
    for char in ['\\', '"', '`', '$']:
        value = value.replace(char, '\\' + char)
    return '"' + value + '"'
data = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
target = data / 'applications/cyoa-manager-catppuccin.desktop'
contents = f'''[Desktop Entry]
Type=Application
Name=CYOA Manager (Catppuccin)
Comment=CYOA library with version history and identified builds
Exec={desktop_arg(launcher)}
Icon={icon}
Terminal=false
Categories=Game;
StartupWMClass=cyoa-manager
Actions=Compatibility;

[Desktop Action Compatibility]
Name=Graphics compatibility mode
Exec={desktop_arg(launcher)} --compatibility
'''
print(target)
print(contents)
if args.apply:
    if not launcher.is_file():
        raise SystemExit('Packaged launcher missing')
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists() and target.read_text() != contents:
        from datetime import datetime
        backups = data / 'cyoa-manager/backups/desktop-shortcuts'
        backups.mkdir(parents=True, exist_ok=True)
        backup = backups / (target.name + '.' + datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '.bak')
        shutil.copy2(target, backup)
    temporary = target.with_suffix('.desktop.tmp')
    temporary.write_text(contents)
    temporary.replace(target)
    if shutil.which('update-desktop-database'):
        subprocess.run(['update-desktop-database', str(target.parent)], check=True)
