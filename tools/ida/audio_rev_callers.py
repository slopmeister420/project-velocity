import idc, idautils, ida_funcs
# callers of every function in [0x4A340, 0x4A680) -> research/audio/review/gui_callers.txt
D = r"D:\projects\project poland rewrite\research\audio\review"
with open(D + r"\gui_callers.txt", "w") as fo:
    for f in idautils.Functions(0x4A300, 0x4A700):
        fo.write("%x %s\n" % (f, idc.get_func_name(f)))
        seen = set()
        for x in idautils.CodeRefsTo(f, 0):
            cf = ida_funcs.get_func(x)
            n = idc.get_func_name(cf.start_ea) if cf else "?"
            if (n, x) in seen:
                continue
            seen.add((n, x))
            fo.write("    %x %s\n" % (x, n))
print("ok")
