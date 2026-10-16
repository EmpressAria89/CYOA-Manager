#!/usr/bin/env python3
"""Launch only a separate test app with corrupt isolated storage; needs a desktop session."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--report', type=Path)
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix='cyoa-startup-failure-') as temporary:
    root = Path(temporary)
    database = root / 'manager/save/library.sqlite3'
    database.parent.mkdir(parents=True)
    database.write_bytes(b'Intentionally invalid SQLite fixture; preserve this original.')
    before = hashlib.sha256(database.read_bytes()).hexdigest()
    environment = os.environ.copy()
    environment.update({'CYOA_MANAGER_DATA_DIR': str(root / 'manager'), 'XDG_DATA_HOME': str(root / 'data'), 'XDG_CONFIG_HOME': str(root / 'config'), 'XDG_CACHE_HOME': str(root / 'cache')})
    with (root / 'app.log').open('w') as log:
        process = subprocess.Popen([str(args.binary.resolve())], env=environment, stdout=log, stderr=subprocess.STDOUT)
        try:
            time.sleep(4)
            alive = process.poll() is None
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
    output = (root / 'app.log').read_text()
    result = {'originalDatabaseUnchanged': hashlib.sha256(database.read_bytes()).hexdigest() == before, 'startupErrorReported': 'Failed to load library storage:' in output, 'appStayedRunningToShowError': alive}
    assert all(result.values()), result
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
