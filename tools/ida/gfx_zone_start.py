import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\zone_start.txt"

def dis(f, s, e):
    a = s
    while a < e:
        f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
        a = idc.next_head(a, e + 16)

with open(OUT, "w", encoding="utf-8") as f:
    f.write("// 162D10 body\n")
    dis(f, 0x162D90, 0x162E80)
    try:
        f.write(str(ida_hexrays.decompile(0x162D10)) + "\n")
    except Exception as ex:
        f.write("// fail %s\n" % ex)
    for name in ["StartZone", "StopZone"]:
        for fa in idautils.Functions():
            n = idc.get_func_name(fa)
            if name in n:
                f.write("// fn %08X %s callers:" % (fa, n))
                for x in idautils.CodeRefsTo(fa, 0):
                    f.write(" %08X(%s)" % (x, idc.get_func_name(x)))
                f.write("\n")
print("ok")
