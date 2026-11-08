# texture lookup by name used by GDL_RelocateModel (dword_77FD30 vtable +20 / +24) -> research/gfx/int3.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
want = set()
for lo, hi in ((0x30AEE0, 0x30E3B0), (0x29E180, 0x2A01A0)):
    for f in idautils.Functions(lo, hi):
        for ea in idautils.FuncItems(f):
            if idc.print_insn_mnem(ea) != "mov" or idc.get_operand_type(ea, 1) != idc.o_imm:
                continue
            v = idc.get_operand_value(ea, 1)
            if not (0x400000 < v < 0x800000):
                continue
            t0 = idc.get_wide_dword(v)
            if not (lo <= t0 < hi):
                continue
            ents = []
            for k in range(8):
                t = idc.get_wide_dword(v + 4 * k)
                ents.append("+%d %X" % (4 * k, t))
            out.append("// %X in %X vtable %X: %s" % (ea, f, v, " ".join(ents)))
            for k in (5, 6):
                t = idc.get_wide_dword(v + 4 * k)
                if 0x11000 < t < 0x3DDD20:
                    want.add(t)
for t in sorted(want):
    try:
        out.append("//==== %X %s\n%s" % (t, idc.get_func_name(t), ida_hexrays.decompile(t)))
    except Exception as ex:
        out.append("// fail %X %s" % (t, ex))
# who writes dword_77FD30
for x in idautils.XrefsTo(0x77FD30):
    if idc.print_insn_mnem(x.frm) == "mov" and idc.get_operand_type(x.frm, 0) == idc.o_mem:
        out.append("// write 77FD30 at %X in %s: %s" % (x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
open(B + r"\int3.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(want))
