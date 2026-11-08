import idc, idautils, ida_funcs
# callers of the addresses in research/audio/xr_req.txt -> research/audio/xr_out.txt
B = r"D:\projects\project poland rewrite\research\audio"
with open(B + r"\xr_out.txt", "w") as fo:
    for line in open(B + r"\xr_req.txt"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        ea = int(line, 16)
        fo.write("== %X %s\n" % (ea, idc.get_func_name(ea)))
        for x in idautils.XrefsTo(ea):
            f = ida_funcs.get_func(x.frm)
            fo.write("  %X in %s\n" % (x.frm, idc.get_func_name(f.start_ea) if f else "?"))
