import idc, ida_bytes, ida_funcs, ida_hexrays, ida_auto
out = open(r"D:/projects/project poland rewrite/research/core_savetable.c", "w")
crcnames = {}
for line in open(r"D:/projects/project poland rewrite/research/strings.tsv", encoding="utf-8", errors="replace"):
    pass
ea = 0x6C1FD0
fns = set()
while True:
    w = [ida_bytes.get_dword(ea + 4 * i) for i in range(9)]
    out.write("// %X node %s\n" % (ea, " ".join("%08X" % x for x in w)))
    if w[0] == 0:
        break
    for x in [w[1]] + w[3:9]:
        if 0x11000 <= x < 0x3E0000:
            fns.add(x)
    ea += 36
for f in sorted(fns):
    if not ida_funcs.get_func(f):
        ida_funcs.add_func(f)
        ida_auto.auto_wait()
    out.write("\n// ===== %X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)))
    except Exception as e:
        out.write("// decompile failed: %s\n" % e)
out.close()
