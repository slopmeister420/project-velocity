import idc, idautils, ida_hexrays, ida_funcs
# attract mode object (byte_6F5A28): decompile FontHelper.cpp region + list xrefs to the object
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_attract.c", "w")
xr = open(base + r"\core_attract_xrefs.txt", "w")
for ea in (0x6F5A28, 0x630F18, 0x749128, 0x749110):
    xr.write("== xrefs %x\n" % ea)
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        xr.write("%x %s\n" % (x.frm, idc.get_func_name(x.frm) if f else "?"))
xr.close()
f = 0xB2BD0
while f < 0xB3C30:
    fn = ida_funcs.get_func(f)
    if fn is None:
        f = idc.next_head(f, 0xB3C30)
        continue
    try:
        out.write("\n==== %x %s\n%s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea), str(ida_hexrays.decompile(fn.start_ea))))
    except Exception as e:
        out.write("\n==== %x fail %s\n" % (fn.start_ea, e))
    f = fn.end_ea
out.close()
