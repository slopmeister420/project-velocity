"""salvage user names from a damaged .i64 whose id0 b-tree (v2) is still intact.

usage: python tools/ida/salvage_names.py <damaged.i64> <out.tsv>
writes `addr<TAB>name` for every '.'+ea(be64)+'N' record reachable from the b-tree root.
plain python, no ida needed (the corruption that makes ida say "database is empty" is in the
section headers, not in the id0 pages)."""
import struct, sys


def main(path, out):
    f = open(path, "rb")
    data = f.read()
    sig = data.find(b"B-tree v2")
    if sig < 0:
        sys.exit("no b-tree v2 signature")
    base = sig - 0x13
    firstfree, pagesize, root, reccount, pagecount = struct.unpack_from("<IHIII", data, base)

    def page(n):
        return data[base + n * pagesize: base + (n + 1) * pagesize]

    names = {}
    seen = set()

    def rec(p, ofs):
        klen, = struct.unpack_from("<H", p, ofs)
        key = p[ofs + 2: ofs + 2 + klen]
        vlen, = struct.unpack_from("<H", p, ofs + 2 + klen)
        val = p[ofs + 4 + klen: ofs + 4 + klen + vlen]
        return key, val

    def take(key, val):
        if len(key) == 10 and key[0:1] == b"." and key[9:10] == b"N":
            ea = struct.unpack(">Q", key[1:9])[0]
            nm = val.rstrip(b"\0")
            if nm and all(32 < c < 127 for c in nm):
                names[ea] = nm.decode()

    def walk(n, depth=0):
        if n in seen or n >= pagecount or depth > 32:
            return
        seen.add(n)
        p = page(n)
        preceding, count = struct.unpack_from("<IH", p, 0)
        if preceding:
            walk(preceding, depth + 1)
            for i in range(count):
                child, recofs = struct.unpack_from("<IH", p, 6 + i * 6)
                key, val = rec(p, recofs)
                take(key, val)
                walk(child, depth + 1)
        else:
            prev = b""
            for i in range(count):
                indent, _unk, recofs = struct.unpack_from("<HHH", p, 6 + i * 6)
                key, val = rec(p, recofs)
                key = prev[:indent] + key
                prev = key
                take(key, val)

    walk(root)
    with open(out, "w") as o:
        for ea in sorted(names):
            o.write("%X\t%s\n" % (ea, names[ea]))
    print("pages", pagecount, "visited", len(seen), "names", len(names))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
