import idc, idautils, ida_funcs
# for each function in [S,E) list callers outside [S,E); writes research/core_ext_callers.txt
S, E = 0x2FE790, 0x328CD0
out = open(r"D:\projects\project poland rewrite\research\core_ext_callers.txt", "w")
for f in idautils.Functions(S, E):
    callers = set()
    for x in idautils.CodeRefsTo(f, 0):
        g = ida_funcs.get_func(x)
        if g and not (S <= g.start_ea < E):
            callers.add((g.start_ea, idc.get_func_name(g.start_ea)))
    for x in idautils.DataRefsTo(f):
        g = ida_funcs.get_func(x)
        if g and not (S <= g.start_ea < E):
            callers.add((g.start_ea, idc.get_func_name(g.start_ea)))
    if callers:
        out.write("%x %s <- %s\n" % (f, idc.get_func_name(f), ", ".join("%x %s" % c for c in sorted(callers))))
out.close()
print("ok")
