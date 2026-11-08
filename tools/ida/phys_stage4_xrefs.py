import idaapi, idautils, idc, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\phys_stage4_xrefs.txt", "w")
def fn(ea):
    f = idaapi.get_func(ea)
    return (f.start_ea if f else 0, idc.get_func_name(ea))
seen = set()
for g in [0x6D245C, 0x6D2458, 0x705F48]:
    out.write("== %X\n" % g)
    for x in idautils.XrefsTo(g):
        s, n = fn(x.frm)
        out.write("  %X in %X %s type %d\n" % (x.frm, s, n, x.type))
        if s: seen.add(s)
for name in ["B::PhysicsManager", "B__PhysicsManager"]:
    ea = idc.get_name_ea_simple(name)
    out.write("name %s %X\n" % (name, ea))
    if ea != idc.BADADDR: seen.add(ea)
for s in sorted(seen):
    out.write("\n// ==== %08X %s\n" % (s, idc.get_func_name(s)))
    try:
        out.write(str(ida_hexrays.decompile(s)))
    except Exception as e:
        out.write("decompile failed %s\n" % e)
out.close()
