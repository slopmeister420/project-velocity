# decompile the screen-fade (flt_7190FC) users; print lines near references
import ida_hexrays, idc, ida_funcs, idautils
out = open("D:/projects/project poland rewrite/research/gfx/s3_fadeusers.c", "w", encoding="utf-8")
seen = set()
for x in idautils.XrefsTo(0x7190FC):
    fn = ida_funcs.get_func(x.frm)
    if not fn:
        out.write("// no func at %X: %s\n" % (x.frm, idc.generate_disasm_line(x.frm, 0)))
        continue
    if fn.start_ea in seen:
        continue
    seen.add(fn.start_ea)
    try:
        lines = str(ida_hexrays.decompile(fn.start_ea)).split("\n")
    except Exception as ex:
        out.write("// fail %X %s\n" % (fn.start_ea, ex))
        continue
    out.write("//==== %X %s (%d lines)\n" % (fn.start_ea, idc.get_func_name(fn.start_ea), len(lines)))
    if len(lines) < 60:
        out.write("\n".join(lines) + "\n")
    else:
        hit = [i for i, l in enumerate(lines) if "7190FC" in l or "719100" in l or "719104" in l or "719108" in l]
        done = -1
        for i in hit:
            for j in range(max(i - 6, done + 1), min(i + 8, len(lines))):
                out.write("%5d %s\n" % (j, lines[j]))
                done = j
            out.write("  ...\n")
for a in (0x4250A0,):
    out.write("// static init %X\n" % a)
    fn = ida_funcs.get_func(0x4250B3)
    if fn:
        out.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
for a in (0xBDEF8, 0xBDF10, 0xBDF31):
    out.write("// %X %s\n" % (a, idc.generate_disasm_line(a, 0)))
out.close()
print("ok")
