import ida_funcs, ida_hexrays, idc, idautils, ida_bytes
# make functions at undefined code in the BINK section gaps and decompile them; disasm 547B60
base = r"D:\projects\project poland rewrite\research"
out = []
for ea in (0x544E00, 0x5450D0):
    if not ida_funcs.get_func(ea):
        idc.create_insn(ea)
        ida_funcs.add_func(ea)
    try:
        out.append("// ---- %X\n%s" % (ea, str(ida_hexrays.decompile(ea))))
    except Exception as e:
        out.append("// ---- %X failed %s" % (ea, e))
# list remaining gaps between functions in BINK
s = 0x53E0C0
prev_end = s
for f in idautils.Functions(0x53E0C0, 0x5500C4):
    if f - prev_end > 16:
        out.append("// GAP %X..%X" % (prev_end, f))
    prev_end = idc.get_func_attr(f, idc.FUNCATTR_END)
dis = []
for f in (0x547B60,):
    for h in idautils.FuncItems(f):
        dis.append("%X %s" % (h, idc.GetDisasm(h)))
open(base + r"\fe_bink_gaps.c", "w").write("\n".join(out))
open(base + r"\fe_bink_dis.txt", "w").write("\n".join(dis))
