#!/usr/bin/env python3
"""Materialize lossless archives for rollback to an older executable. ZIPs are retained."""
import argparse,hashlib,json,os,shutil,tempfile,zipfile
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--data-dir',type=Path,default=Path(os.environ.get('XDG_DATA_HOME',Path.home()/'.local/share'))/'cyoa-manager')
parser.add_argument('--apply',action='store_true')
args=parser.parse_args()
for archive in sorted((args.data_dir/'save/versions').glob('*/*/files.zip')):
    files=archive.parent/'files'
    if files.exists():
        print(f'Already extracted: {archive.parent.name}')
        continue
    print(f'Extract: {archive.parent.name}')
    if not args.apply:
        continue
    expected=json.loads((archive.parent/'checksums.json').read_text())
    temporary=Path(tempfile.mkdtemp(prefix='.rollback-',dir=archive.parent))
    try:
        found={}
        with zipfile.ZipFile(archive) as source:
            for entry in source.infolist():
                path=Path(entry.filename)
                if path.is_absolute() or '..' in path.parts:
                    raise ValueError('Invalid archive path')
                if entry.is_dir():
                    continue
                target=temporary/path
                target.parent.mkdir(parents=True,exist_ok=True)
                with source.open(entry) as data,target.open('wb') as output:
                    shutil.copyfileobj(data,output)
                name=path.as_posix()
                if name in found:
                    raise ValueError('Duplicate archive path')
                with target.open('rb') as data:
                    found[name]=hashlib.file_digest(data,'sha256').hexdigest()
        if found!=expected:
            raise ValueError('Archive checksum mismatch')
        temporary.rename(files)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)
