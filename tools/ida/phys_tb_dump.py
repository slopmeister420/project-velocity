import ida_hexrays, ida_funcs, idc
# dump score-manager gear-switch readers to research/phys_gs.c
ADDRS = [0x16AFE0, 0x16CD50, 0x16CD70]
out = []
for a in ADDRS:
    try:
        c = str(ida_hexrays.decompile(a))
    except Exception as e:
        c = "error %s" % e
    out.append("// ==== %X %s\n%s\n" % (a, idc.get_func_name(a), c))
    for x in __import__("idautils").XrefsTo(a):
        out.append("// xref %X %s" % (x.frm, idc.get_func_name(x.frm)))
open(r"D:\projects\project poland rewrite\research\phys_gs.c", "w").write("\n".join(out))
print("ok")
