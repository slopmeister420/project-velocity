# frontend agent: option screen tables + userinput helpers -> research/fe_opt_q2.txt
import idautils, idc, ida_hexrays, ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_opt_q2.txt", "w", encoding="utf-8")
def tbl(base, n):
    out.write("== table %08X x%d\n" % (base, n))
    for i in range(n):
        g = base + 4 * i
        for x in idautils.XrefsTo(g):
            p = x.frm
            name = None
            for _ in range(12):
                p = idc.prev_head(p)
                d = idc.GetDisasm(p)
                if "push    offset" in d:
                    name = d
                    break
            out.write("  [%d] %08X <- %08X %s | %s\n" % (i, g, x.frm, name, idc.GetDisasm(x.frm)))
for b, n in [(0x766940, 7), (0x766168, 4), (0x766400, 3), (0x766698, 4), (0x765E70, 8)]:
    tbl(b, n)
out.write("== off_6C1850\n")
for i in range(10):
    p = ida_bytes.get_dword(0x6C1850 + 4 * i)
    out.write("  %d %s\n" % (i, idc.get_strlit_contents(p)))
out.write("== 6C17B0 positions\n")
for i in range(10):
    x = struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(0x6C17B0 + 8 * i)))[0]
    y = struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(0x6C17B4 + 8 * i)))[0]
    out.write("  %d %g %g\n" % (i, x, y))
for a in [0x42F40, 0x1E7090, 0x1E7140, 0x1E71F0, 0x1E7360, 0x1E6F80, 0x27C320, 0x27C420, 0x2793A0, 0x27AA90, 0x26FB60, 0x271E80, 0xBC3D0]:
    out.write("// ===== %08X %s\n" % (a, idc.get_func_name(a)))
    try:
        out.write(str(ida_hexrays.decompile(a)) + "\n")
    except Exception as e:
        out.write("fail %s\n" % e)
out.close()
print("ok")
