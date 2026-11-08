# fury constants + caller sites -> research/gfx/fury_vals.txt
import idautils, ida_funcs, idc, ida_bytes, struct
out = "D:/projects/project poland rewrite/research/gfx/fury_vals.txt"
with open(out, "w", encoding="utf-8") as f:
    for a in range(0x686B5C, 0x686B94, 4):
        d = ida_bytes.get_dword(a)
        f.write("%08X %08X %r\n" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
    for a in [0x71CEA0, 0x71CEA4, 0x71A078, 0x6F4AC8]:
        f.write("%08X %08X seg=%s\n" % (a, ida_bytes.get_dword(a), idc.get_segm_name(a)))
    for t in [0xBFAD0, 0xBFB70, 0x128590, 0x1285E0, 0x128530, 0x128100, 0x1287B0]:
        f.write("== callers of %08X\n" % t)
        for x in idautils.XrefsTo(t):
            fn = ida_funcs.get_func(x.frm)
            f.write("  %08X %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
    # context around callers in GameManager: 6 insns before each
    f.write("== context\n")
    for t in [0xBFAD0, 0xBFB70, 0x128590, 0x1285E0]:
        for x in idautils.XrefsTo(t):
            ea = x.frm
            p = ea
            lines = []
            for i in range(8):
                p = idc.prev_head(p)
                lines.append("    %08X %s" % (p, idc.GetDisasm(p)))
            f.write("-- %08X\n" % ea + "\n".join(reversed(lines)) + "\n")
print("ok")
