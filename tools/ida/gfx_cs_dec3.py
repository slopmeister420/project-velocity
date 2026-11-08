# fixed-function car sets: the "A1" setter, texmgr +4, override use -> research/gfx/cs_dec3.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
t = idc.get_operand_value(0x1564F4, 0)
out.append("// A1 target %X %s" % (t, idc.get_func_name(t)))
for a in (t, idc.get_name_ea_simple("CShaderManagerBase::SetZState")):
    out.append("//==== %X %s" % (a, idc.get_func_name(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
# texture manager +4 writers (dword_77FD18 object): look for the ctor/assignments via xrefs of 11F260
for x in idautils.XrefsTo(0x11F260):
    out.append("// 11F260 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
# disasm around the odd 156150 xrefs
for a in (0x15706A, 0x1568EF):
    ea = a
    for k in range(12):
        ea = idc.prev_head(ea)
    lines = []
    for k in range(16):
        lines.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
    out.append("// around %X (func %s):\n// " % (a, idc.get_func_name(a)) + "\n// ".join(lines))
open(B + r"\cs_dec3.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
