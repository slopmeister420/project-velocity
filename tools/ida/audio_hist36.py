import idc, ida_funcs, ida_hexrays, idautils, re
# callers of VsConfigFileRead__1B6E80 (VE input history entry): lines that write entry +36 / +44 / +24
OUT = r"D:\projects\project poland rewrite\research\audio\hist36.txt"
ida_hexrays.init_hexrays_plugin()
seen = set()
with open(OUT, "w", encoding="utf-8") as fo:
    for x in idautils.CodeRefsTo(0x1B6E80, 0):
        f = ida_funcs.get_func(x)
        if not f or f.start_ea in seen:
            continue
        seen.add(f.start_ea)
        try:
            c = ida_hexrays.decompile(f.start_ea)
        except Exception:
            c = None
        fo.write("== %x %s\n" % (f.start_ea, idc.get_func_name(f.start_ea)))
        if c is None:
            continue
        for ln in str(c).splitlines():
            if re.search(r"1B6E80|\+ 36\) =|\+ 9\) =|\+ 36\);", ln):
                fo.write("  " + ln.strip()[:220] + "\n")
