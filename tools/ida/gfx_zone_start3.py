import idc, idautils, ida_bytes, ida_search, ida_ida
OUT = r"D:\projects\project poland rewrite\research\gfx\zone_start3.txt"
pat = "E1 5B 8C 67"
with open(OUT, "w", encoding="utf-8") as f:
    ea = ida_ida.inf_get_min_ea()
    end = ida_ida.inf_get_max_ea()
    n = 0
    while n < 50:
        ea = ida_bytes.find_bytes(pat, ea + 1, end)
        if ea == idc.BADADDR or ea is None or ea >= end:
            break
        f.write("%08X %s\n" % (ea, idc.get_func_name(ea)))
        n += 1
    for g in [0x6D4A94, 0x687524]:
        f.write("// refs %08X:" % g)
        for x in idautils.DataRefsTo(g):
            f.write(" %08X(%s)" % (x, idc.get_func_name(x)))
        f.write("\n")
print("ok")
