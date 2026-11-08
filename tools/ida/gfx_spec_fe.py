# find the class of FEAlloyEditor_Rim 0x285350 (vtable -> ctor) and decompile the ctor + functions using +764 (dword 191)
import idautils, idc, ida_funcs, ida_hexrays, re
out = r"D:/projects/project poland rewrite/research/gfx/spec_fe.c"
with open(out, "w", encoding="utf-8") as f:
    vts = [x.frm for x in idautils.XrefsTo(0x285350)]
    f.write("// refs to 285350: %s\n" % ", ".join("%08X" % v for v in vts))
    seen = set()
    for v in vts:
        # walk back to the vtable start (previous dword with a name)
        start = v
        for k in range(64):
            if idc.get_name(start - 4 * k):
                start = start - 4 * k
                break
        f.write("// vtable %08X %s (slot %d)\n" % (start, idc.get_name(start), (v - start) // 4))
        for x in idautils.XrefsTo(start):
            fn = ida_funcs.get_func(x.frm)
            if not fn or fn.start_ea in seen:
                continue
            seen.add(fn.start_ea)
            f.write("//==== ctor? %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
            try:
                s = str(ida_hexrays.decompile(fn.start_ea))
            except Exception as ex:
                s = "// fail %s" % ex
            f.write(s + "\n")
            # callers of the ctor
            for y in idautils.XrefsTo(fn.start_ea):
                g = ida_funcs.get_func(y.frm)
                f.write("//   called from %08X %s\n" % (y.frm, idc.get_func_name(g.start_ea) if g else "?"))
print("wrote", out)
