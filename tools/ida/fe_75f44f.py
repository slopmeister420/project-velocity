import idautils, idc, ida_funcs
out = []
for x in idautils.DataRefsTo(0x75F44F):
    f = ida_funcs.get_func(x)
    out.append("%X %s | %s" % (x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
    p = idc.prev_head(x); out.append("   prev: " + idc.generate_disasm_line(p, 0))
open(r"D:/projects/project poland rewrite/research/fe_75f44f.txt", "w").write("\n".join(out))
