import idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\core_s6\vt.txt"
out = []
def fn(a):
    f = ida_funcs.get_func(a)
    return "%X %s" % (f.start_ea, idc.get_func_name(f.start_ea)) if f else "%X ?" % a
for a in [0x179E70, 0x17F380, 0x178120]:
    for x in idautils.DataRefsTo(a):
        out.append("dref %X -> %s" % (x, fn(a)))
        # dump the vtable around it
        base = x - 0x20
        for k in range(16):
            v = idc.get_wide_dword(base + 4 * k)
            out.append("   %X [+%d] %s" % (base + 4 * k, 4 * k - 0x20, fn(v)))
    for x in idautils.CodeRefsTo(a, 0):
        out.append("cref %X from %s" % (x, fn(x)))
# peer vtable
for k in range(12):
    v = idc.get_wide_dword(0x640218 + 4 * k)
    out.append("peer +%X %s" % (4 * k, fn(v)))
open(OUT, "w").write("\n".join(out))
print("ok")
