"""Writes a private single-file torrent and prints its info hash.

usage: make_torrent.py <payload> <announce_url> <output.torrent>
"""

import hashlib
import os
import sys

PIECE_LENGTH = 256 * 1024


def bencode(value):
    if isinstance(value, int):
        return b"i%de" % value
    if isinstance(value, str):
        value = value.encode()
    if isinstance(value, bytes):
        return b"%d:%s" % (len(value), value)
    if isinstance(value, dict):
        return b"d" + b"".join(bencode(k) + bencode(value[k]) for k in sorted(value)) + b"e"
    raise TypeError(value)


payload, announce, output = sys.argv[1:]
with open(payload, "rb") as f:
    pieces = b"".join(hashlib.sha1(chunk).digest() for chunk in iter(lambda: f.read(PIECE_LENGTH), b""))
info = {
    "length": os.path.getsize(payload),
    "name": os.path.basename(payload),
    "piece length": PIECE_LENGTH,
    "pieces": pieces,
    "private": 1,
}
with open(output, "wb") as f:
    f.write(bencode({"announce": announce, "info": info}))
print(hashlib.sha1(bencode(info)).hexdigest())
