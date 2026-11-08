import idautils, idc, ida_funcs
out = []
for ea in range(0x6F5C14, 0x6F5C84, 4):
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        fn = idc.get_func_name(x.frm) if f else "?"
        out.append("%X <- %X %s %s" % (ea, x.frm, fn, idc.generate_disasm_line(x.frm, 0)))
open(r"D:\projects\project poland rewrite\research\ai_startline_xrefs.txt", "w").write("\n".join(out))
