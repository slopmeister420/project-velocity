# fixed-function car sets: set objects, per-frame users -> research/gfx/cs_dec2.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
addrs = [0x158A90, 0x158C70, 0x158CB0, 0x158CF0, 0x158D30, 0x2A0AA0, 0x2A0F40, 0x2A1140,
         0x1588C0, 0x158940, 0x1589A0, 0x156520, 0x156C00, 0x1577F0, 0x156680]
for a in addrs:
    f = ida_funcs.get_func(a)
    if not f:
        out.append("// nofunc %X" % a)
        continue
    out.append("//==== %X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
# call targets inside 1564A0 / 156520 / 1577F0 (find the "A1" helper address)
for a in (0x1564A0, 0x156520, 0x1577F0):
    f = ida_funcs.get_func(a)
    ea = f.start_ea
    lines = []
    while ea < f.end_ea:
        m = idc.print_insn_mnem(ea)
        if m in ("call", "push"):
            lines.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
    out.append("// disasm %X:\n// " + "\n// ".join(lines))
for t in (0x1588C0, 0x158940, 0x1589A0):
    for x in idautils.XrefsTo(t):
        out.append("// %X caller %X in %s" % (t, x.frm, idc.get_func_name(x.frm)))
open(B + r"\cs_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
