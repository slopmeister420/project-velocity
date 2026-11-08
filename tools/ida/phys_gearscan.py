# physics agent: transmission setting readers (dword_768978+2220) and manual-gear flag writers (VE+0xDD0 / car+0xCE0)
import idautils, idc, ida_ua, ida_funcs
DS = (0x8AC, 0xDD0, 0xCE0)
out = []
for f in idautils.Functions(0x11000, 0x3C63D0):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and (op.addr & 0xFFFFFFFF) in DS:
                out.append("%X %s %s" % (ea, idc.get_func_name(f), idc.GetDisasm(ea)))
out.append("---- refs to 0x768978")
for x in idautils.DataRefsTo(0x768978):
    out.append("%X %s %s" % (x, idc.get_func_name(x), idc.GetDisasm(x)))
open(r"D:\projects\project poland rewrite\research\phys_gearscan.txt", "w").write("\n".join(out))
print(len(out))
