# frontend agent: hex dump of ranges in fe_hex_args.txt (start len) -> research/fe_hex.txt
import ida_bytes
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_hex_args.txt").read().split()
with open(r"D:\projects\project poland rewrite\research\fe_hex.txt", "w") as o:
    for i in range(0, len(a), 2):
        s, n = int(a[i], 16), int(a[i+1], 16)
        b = ida_bytes.get_bytes(s, n) or b""
        for k in range(0, len(b), 16):
            c = b[k:k+16]
            o.write("%08X %s %s\n" % (s + k, c.hex(" "), "".join(chr(x) if 32 <= x < 127 else "." for x in c)))
