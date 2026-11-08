import idautils, idc, ida_funcs, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\core_camlerp2.txt", "w")
def name(ea):
    return idc.get_func_name(ea) or hex(ea)
def callers(t):
    r = []
    for x in idautils.CodeRefsTo(t, 0):
        s = idc.get_func_attr(x, idc.FUNCATTR_START)
        r.append((x, s))
    return r
def dec(f):
    try:
        out.write("\n==== %x %s\n%s\n" % (f, name(f), str(ida_hexrays.decompile(f))))
    except Exception as e:
        out.write("\n==== %x fail %s\n" % (f, e))
for t in [0x5FA80, 0x5FB30, 0x1C57A0, 0x1C57E0, 0x57580]:
    out.write("== callers of %s\n" % name(t))
    for x, s in callers(t):
        out.write("  %x in %s\n" % (x, name(s)))
for f in [0x1C57A0, 0x1C57E0]:
    dec(f)
out.close()
