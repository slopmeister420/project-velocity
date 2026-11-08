# physics agent: speedo / nos getters -> research/phys_dump/speedo.txt
import idc, ida_ua, ida_funcs, ida_hexrays, idautils

OUT = r"D:\projects\project poland rewrite\research\phys_dump\speedo.txt"
with open(OUT, "w") as o:
    for ea in (0x000F78D0, 0x000F7970, 0x000F79A0, 0x00024720, 0x000248F0):
        f = ida_funcs.get_func(ea)
        o.write("==== %08X %s\n" % (ea, idc.get_func_name(ea)))
        a = f.start_ea
        while a < f.end_ea:
            o.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
            n = ida_ua.decode_insn(ida_ua.insn_t(), a)
            a += n if n > 0 else 1
        o.write("-- callers:")
        for x in idautils.CodeRefsTo(ea, 0):
            o.write(" %08X(%s)" % (x, idc.get_func_name(x)))
        o.write("\n")
    # GetRatioForGauge
    try:
        o.write(str(ida_hexrays.decompile(0x000F4F40)))
    except Exception as e:
        o.write("decompile failed %s\n" % e)
print("ok")
