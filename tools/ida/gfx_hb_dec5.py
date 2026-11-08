# sort used by VsRender 0x5A6C0 -> research/gfx/hb_dec5.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
todo = [0x5B2E0, 0x5B1D0]
while todo and len(seen) < 12:
    ea = todo.pop(0)
    f = ida_funcs.get_func(ea)
    if not f or f.start_ea in seen:
        continue
    seen.add(f.start_ea)
    out.append("//==== %08X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
    # follow callees inside the VsRender range (sort helpers)
    for h in idautils.FuncItems(f.start_ea):
        for x in idautils.CodeRefsFrom(h, 0):
            if 0x57000 <= x < 0x5E000 and x not in seen:
                todo.append(x)
open(B + r"\hb_dec5.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(seen))
