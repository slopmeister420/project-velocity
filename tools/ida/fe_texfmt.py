# frontend agent: movie texture format (Register "movie_test" fmt 18 -> D3DFORMAT) -> research/fe_texfmt.c
import idc, ida_hexrays, ida_name, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_texfmt.c", "w", encoding="utf-8")
def dc(f, depth):
    f = idc.get_func_attr(f, idc.FUNCATTR_START)
    if f in seen or f == idc.BADADDR:
        return
    seen.add(f)
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
    if depth > 0:
        for ea in idautils.FuncItems(f):
            for x in idautils.CodeRefsFrom(ea, 0):
                if idc.get_func_attr(x, idc.FUNCATTR_START) == x and 0x11000 <= x < 0x3DDD20:
                    dc(x, depth - 1)
seen = set()
ea = ida_name.get_name_ea(idc.BADADDR, "CTextureManager::Register")
out.write("// Register at %08X\n" % ea)
dc(ea, 2)
out.close()
