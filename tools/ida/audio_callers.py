import idautils, idc, ida_funcs
# who calls the src/Audio entry points from outside the audio code
lo, hi = 0x47910, 0x56D90
out = []
for f in idautils.Functions(lo, hi):
    callers = set()
    for x in idautils.CodeRefsTo(f, 0):
        g = ida_funcs.get_func(x)
        if g and not (lo <= g.start_ea < hi):
            callers.add(idc.get_func_name(g.start_ea))
    if callers:
        out.append("%x %s <- %s" % (f, idc.get_func_name(f), ", ".join(sorted(callers))[:400]))
print("\n".join(out))
