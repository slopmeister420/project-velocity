import ida_hexrays, idc, idautils, ida_funcs
# decompile the functions listed in research/core_dc_in.txt (hex addrs) into research/core_dc_out.c,
# with each function's callers appended as a comment
base = r"D:\projects\project poland rewrite\research"
addrs = [int(l.split()[0], 16) for l in open(base + r"\core_dc_in.txt") if l.strip()]
out = []
for a in addrs:
    callers = sorted({ida_funcs.get_func(x).start_ea for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)})
    drefs = [x for x in idautils.DataRefsTo(a)]
    hdr = "// ---- %X %s callers %s drefs %s" % (a, idc.get_func_name(a), " ".join("%X" % c for c in callers),
                                                " ".join("%X" % d for d in drefs))
    try:
        out.append(hdr + "\n" + str(ida_hexrays.decompile(a)))
    except Exception as e:
        out.append(hdr + " failed %s" % e)
open(base + r"\core_dc_out.c", "w").write("\n".join(out))
