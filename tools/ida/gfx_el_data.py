# values + referencing functions of the addresses in research/gfx/el_data_in.txt -> el_data_out.txt
import idautils, idc, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_data_in.txt"):
    l = l.split("#")[0].strip()
    if not l:
        continue
    p = l.split()
    a = int(p[0], 16)
    n = int(p[1]) if len(p) > 1 else 1
    for k in range(n):
        ea = a + 4 * k
        d = ida_bytes.get_dword(ea)
        f = struct.unpack("<f", struct.pack("<I", d))[0]
        refs = set()
        for x in idautils.XrefsTo(ea, 0):
            fn = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
            refs.add("%s%s" % (idc.get_func_name(fn) if fn != idc.BADADDR else "%X" % x.frm, "w" if x.type == 2 else ""))
        out.append("%X %-10s %08X %r  %s" % (ea, idc.get_name(ea), d, f, " ".join(sorted(refs))[:300]))
open(B + r"\el_data_out.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
