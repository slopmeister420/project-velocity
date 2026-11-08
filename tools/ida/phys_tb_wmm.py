import idautils, idc, ida_funcs, ida_hexrays
# physics agent: who reads dword_6FA8D0 (+0x10 time stretch); decompile BF670 and the readers
out = []
fs = set()
for x in idautils.XrefsTo(0x6FA8D0):
    out.append("x %X %s %s" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
    f = ida_funcs.get_func(x.frm)
    if f:
        fs.add(f.start_ea)
fs.add(0xBF670)
for a in sorted(fs):
    if a in (0xC4870, 0xC4E50, 0xC4FF0):
        continue
    try:
        c = str(ida_hexrays.decompile(a))
    except Exception as e:
        c = "error %s" % e
    out.append("// ==== %X %s\n%s\n" % (a, idc.get_func_name(a), c[:3000]))
open(r"D:\projects\project poland rewrite\research\phys_tb_wmm.c", "w").write("\n".join(out))
print(len(fs))
