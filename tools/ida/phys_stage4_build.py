import idaapi, idautils, idc, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\phys_stage4_build.txt", "w")
seen = set()
for name in ["FCQuadTree::Build", "FCQuadTree::Compact", "FCMesh::ComputeNormals"]:
    ea = idc.get_name_ea_simple(name)
    out.write("== %s %X\n" % (name, ea))
    for x in idautils.CodeRefsTo(ea, 0):
        s = idc.get_func_attr(x, idc.FUNCATTR_START)
        out.write("  call %X in %X %s\n" % (x, s, idc.get_func_name(s)))
        if name == "FCQuadTree::Build":
            seen.add(s)
    seen.add(ea)
for s in [0x1B3F20, 0x1B3F40, 0x7F8C0, 0x7FCA0, 0x132B0, 0x133A0]:
    seen.add(s)
for s in sorted(seen):
    out.write("\n// ==== %08X %s\n" % (s, idc.get_func_name(s)))
    try:
        out.write(str(ida_hexrays.decompile(s)))
    except Exception as e:
        out.write("decompile failed %s\n" % e)
out.close()
