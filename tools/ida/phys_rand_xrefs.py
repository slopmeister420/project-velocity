import idautils, idc, ida_funcs
out = []
seen = set()
def callers(ea, depth):
    for x in idautils.CodeRefsTo(ea, 0):
        f = ida_funcs.get_func(x)
        fs = f.start_ea if f else 0
        out.append("%s%08X -> %08X  at %08X  %s" % ("  " * depth, ea, fs, x, idc.get_func_name(fs)))
        # follow thin wrappers (tiny functions) one level
        if f and (f.end_ea - f.start_ea) < 0x40 and fs not in seen and depth < 2:
            seen.add(fs)
            callers(fs, depth + 1)
for ea in (0x3DE030, 0x13510):
    callers(ea, 0)
with open(r"D:\projects\project poland rewrite\research\phys_rand_xrefs.txt", "w") as fh:
    fh.write("\n".join(out))
