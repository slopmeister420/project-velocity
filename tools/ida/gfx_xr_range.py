# code/data refs to every address in a range -> research/gfx/xr.out
import idautils, ida_funcs, idc, os
S, E = 0x700B00, 0x700D60
out = []
for a in range(S, E):
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        out.append("%08X <- %08X %s" % (a, x.frm, idc.get_func_name(f.start_ea) if f else "-"))
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "research", "gfx", "xr.out"), "w").write("\n".join(out) + "\n")
print("ok")
