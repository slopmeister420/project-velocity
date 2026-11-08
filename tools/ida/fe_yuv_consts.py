# blitter constants (qword_7D2A30..7D2A48, 7D2960..7D29C8) and the segment they live in
import idc, ida_segment
out = r"D:/projects/project poland rewrite/research/fe_yuv_consts.txt"
with open(out, "w", encoding="utf-8") as f:
    for a in list(range(0x7D2960, 0x7D29D0, 8)) + list(range(0x7D2A30, 0x7D2A50, 8)):
        s = ida_segment.getseg(a)
        f.write("%08X %016X %s\n" % (a, idc.get_qword(a), ida_segment.get_segm_name(s) if s else "?"))
    for k in range(6):
        f.write("sel %d %08X\n" % (k, idc.get_wide_dword(0x7D2A60 + 4 * k)))
    # code refs writing the constants
    import idautils
    for a in (0x7D2A30, 0x7D2A38, 0x7D2A40, 0x7D2A48):
        for x in idautils.DataRefsTo(a):
            f.write("ref %08X <- %08X %s %s\n" % (a, x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
print("wrote", out)
