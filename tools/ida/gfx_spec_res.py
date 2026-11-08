# decompile the SpecularChars resolver ctor chain and its vtable functions
import idautils, idc, ida_funcs, ida_hexrays, ida_bytes
out = r"D:/projects/project poland rewrite/research/gfx/spec_res.c"
fns = [0x79C30, 0x143140, 0x79940, 0x7A750, 0x7A7C0]
done = set()
def dc(f, ea):
    fn = ida_funcs.get_func(ea)
    if not fn or fn.start_ea in done:
        return None
    done.add(fn.start_ea)
    f.write("\n//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
    try:
        s = str(ida_hexrays.decompile(fn.start_ea))
    except Exception as ex:
        s = "// fail %s" % ex
    f.write(s + "\n")
    return fn
with open(out, "w", encoding="utf-8") as f:
    for a in fns:
        dc(f, a)
    # vtables referenced by the ctors (off_62B0E0 from 7A750 and any offset written in 79C30's callees)
    vts = set([0x62B0E0])
    for a in (0x79C30, 0x7A750):
        fn = ida_funcs.get_func(a)
        for h in idautils.Heads(fn.start_ea, fn.end_ea):
            for x in idautils.DataRefsFrom(h):
                if 0x600000 <= x < 0x680000:
                    vts.add(x)
    for vt in sorted(vts):
        f.write("\n// vtable %08X %s\n" % (vt, idc.get_name(vt)))
        for i in range(8):
            p = ida_bytes.get_dword(vt + 4 * i)
            if not (0x11000 <= p < 0x3E0000):
                break
            f.write("//  [%d] %08X %s\n" % (i, p, idc.get_func_name(p)))
            dc(f, p)
print("wrote", out)
