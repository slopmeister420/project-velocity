# dynamic envmap: DangerZoneRender 0x1214A0 + callers + callees -> research/gfx/envmap_dec.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
f = ida_funcs.get_func(0x1214A0)
out.append("// func %X..%X %s" % (f.start_ea, f.end_ea, idc.get_func_name(f.start_ea)))
for x in idautils.XrefsTo(f.start_ea):
    out.append("// caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
callees = []
ea = f.start_ea
while ea < f.end_ea:
    if idc.print_insn_mnem(ea) == "call":
        t = idc.get_operand_value(ea, 0)
        n = idc.get_func_name(t)
        callees.append("%X -> %X %s" % (ea, t, n))
    ea = idc.next_head(ea)
out.append("// callees:\n// " + "\n// ".join(callees))
try:
    out.append(str(ida_hexrays.decompile(f.start_ea)))
except Exception as ex:
    out.append("// fail %s" % ex)
for x in idautils.XrefsTo(0x6F7157):
    out.append("// 6F7157 xref %X in %s : %s" % (x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
open(B + r"\envmap_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
