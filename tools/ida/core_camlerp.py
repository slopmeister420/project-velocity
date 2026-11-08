import idautils, idc, ida_funcs, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\core_camlerp.txt", "w")
def fn(ea):
    f = ida_funcs.get_func(ea)
    return f.start_ea if f else None
def name(ea):
    return idc.get_func_name(ea) or hex(ea)
targets = [0x5D250, 0x5D540, 0x5D2E0]
seen = set()
for t in targets:
    out.write("== callers of %s %x\n" % (name(t), t))
    for x in idautils.CodeRefsTo(t, 0):
        f = fn(x)
        out.write("  %x in %s %s\n" % (x, hex(f) if f else "?", name(x)))
        if f: seen.add(f)
for f in sorted(seen):
    try:
        c = ida_hexrays.decompile(f)
        out.write("\n\n==== %x %s\n%s\n" % (f, name(f), str(c)))
    except Exception as e:
        out.write("\n==== %x fail %s\n" % (f, e))
out.close()
