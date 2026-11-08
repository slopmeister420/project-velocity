import idc, idautils, ida_funcs, ida_hexrays
OUT = r"D:\projects\project poland rewrite\research\core_s6\q.txt"
out = []
def fn(a):
    f = ida_funcs.get_func(a)
    return "%X %s" % (f.start_ea, idc.get_func_name(f.start_ea)) if f else "%X ?" % a
for nm in ["GDL_LoadAnim", "VsStringAnsi::Hash", "LoadModelGeometry"]:
    a = idc.get_name_ea_simple(nm)
    out.append("%s = %X" % (nm, a))
    if a != idc.BADADDR:
        try:
            out.append(str(ida_hexrays.decompile(a)))
        except Exception as e:
            out.append("ERR %s" % e)
for a in [0x171B30, 0x1733A0, 0x173380, 0x1735C0, 0x173310, 0x173550]:
    out.append("== callers of %s: %s" % (fn(a), " ".join(sorted({fn(x) for x in idautils.CodeRefsTo(a, 0)}))))
for a in [0x171B30]:
    out.append(str(ida_hexrays.decompile(a)))
open(OUT, "w").write("\n".join(out))
print("ok")
