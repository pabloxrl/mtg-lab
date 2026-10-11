"""Deterministically package raw measurement evidence and bind every file hash."""
import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile


def pack(root, destination, index):
    root=Path(root); destination=Path(destination); members={}
    with destination.open('wb') as output, gzip.GzipFile(fileobj=output,mode='wb',filename='',mtime=0) as zipped:
        with tarfile.open(fileobj=zipped,mode='w') as archive:
            for path in sorted(root.rglob('*')):
                if path.is_dir(): continue
                if path.is_symlink(): raise ValueError('evidence must not follow symlinks')
                data=path.read_bytes();name=path.relative_to(root).as_posix()
                members[name]=dict(bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
                info=tarfile.TarInfo(name);info.size=len(data);info.mode=0o600
                archive.addfile(info,io.BytesIO(data))
    Path(index).write_text(json.dumps(dict(schema_version=1,files=members,
        archive_sha256=hashlib.sha256(destination.read_bytes()).hexdigest()),indent=2,sort_keys=True)+'\n')


def unpack_verified(source, index, destination):
    """Read archived evidence without trusting archive paths or unindexed files."""
    source=Path(source); index=json.loads(Path(index).read_text()); root=Path(destination)
    if index['schema_version']!=1 or hashlib.sha256(source.read_bytes()).hexdigest()!=index['archive_sha256']:
        raise ValueError('archive identity mismatch')
    with tarfile.open(source,'r:gz') as archive:
        members=archive.getmembers()
        if len(members)!=len(index['files']) or {m.name for m in members}!=set(index['files']):
            raise ValueError('missing, extra or duplicate archive member')
        for member in members:
            path=Path(member.name);expected=index['files'][member.name]
            if not member.isfile() or path.is_absolute() or '..' in path.parts or member.size!=expected['bytes']:
                raise ValueError('invalid artifact member')
            data=archive.extractfile(member).read()
            if hashlib.sha256(data).hexdigest()!=expected['sha256']:
                raise ValueError('artifact content mismatch')
            target=root/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
