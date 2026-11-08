# texture-by-name lookup of dword_77FD30 (ctor 0x18F6C0): vtable slots +20 / +24 and callees -> research/gfx/int4.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
f = ida_funcs.get_func(0x18F6C0)
vt = None
for ea in idautils.FuncItems(f.start_ea):
    if idc.print_insn_mnem(ea) == "mov" and idc.get_operand_type(ea, 1) == idc.o_imm and 0x400000 < idc.get_operand_value(ea, 1) < 0x800000 and 0x11000 < idc.get_wide_dword(idc.get_operand_value(ea, 1)) < 0x3DDD20:
        vt = idc.get_operand_value(ea, 1)
        break
out.append("// vtable %X" % vt)
todo = []
for k in range(10):
    t = idc.get_wide_dword(vt + 4 * k)
    out.append("// +%d %X %s" % (4 * k, t, idc.get_func_name(t)))
    if k in (5, 6):
        todo.append(t)
seen = set()
depth = 0
while todo and depth < 12:
    t = todo.pop(0)
    if t in seen:
        continue
    seen.add(t)
    depth += 1
    try:
        c = ida_hexrays.decompile(t)
        out.append("//==== %X %s\n%s" % (t, idc.get_func_name(t), c))
        for ea in idautils.FuncItems(t):
            if idc.print_insn_mnem(ea) == "call":
                d = idc.get_operand_value(ea, 0)
                if 0x11000 < d < 0x3DDD20 and len(seen) + len(todo) < 12:
                    todo.append(d)
    except Exception as ex:
        out.append("// fail %X %s" % (t, ex))
open(B + r"\int4.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(seen))
