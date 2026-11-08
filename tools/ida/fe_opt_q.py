# frontend agent: xrefs for option globals + small decompiles, written to research/fe_opt_q.txt
import idautils, idc, ida_hexrays, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\fe_opt_q.txt", "w", encoding="utf-8")
for g in [0x6F9160, 0x6F915D, 0x6F915E, 0x6F915F, 0x6F5C00, 0x6F9164, 0x6F9168, 0x6F916C, 0x6F9170, 0x6F9174, 0x6F9178, 0x6F917C]:
    out.write("== xrefs %08X\n" % g)
    for x in idautils.XrefsTo(g):
        f = ida_funcs.get_func(x.frm)
        out.write("  %08X in %s  | %s\n" % (x.frm, idc.get_func_name(x.frm) if f else "?", idc.GetDisasm(x.frm)))
for a in [0x4D9B0, 0x4D9D0, 0x4D9F0, 0x4DA10]:
    out.write("// ===== %08X %s\n" % (a, idc.get_func_name(a)))
    try:
        out.write(str(ida_hexrays.decompile(a)) + "\n")
    except Exception as e:
        out.write("fail %s\n" % e)
out.close()
print("ok")
