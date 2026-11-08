import idautils, idc, ida_ua, ida_funcs
# physics agent: xrefs to dword_688B70 / dword_686B58 and byte reads at disp 0x10 in VehicleEntity code
out = []
for g in (0x688B70, 0x686B58):
    for x in idautils.XrefsTo(g):
        out.append("%X x %X %s %s" % (g, x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
for f in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        d = idc.GetDisasm(ea)
        if "byte ptr [" in d and "+10h]" in d and ("cmp" in d or "movzx" in d or "mov" in d):
            n = idc.get_func_name(f)
            if "VehicleEntity" in n or "Physics" in n or "Vehicle" in n:
                out.append("B10 %X %s %s" % (ea, n, d))
open(r"D:\projects\project poland rewrite\research\phys_tb_xrefs.txt", "w").write("\n".join(out))
print(len(out))
