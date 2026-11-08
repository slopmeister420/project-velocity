# the six blitter selectors at off_7D2A60 (sub_549570) and what they install into dword_7D2A0C
import idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/fe_yuv_sel.txt"
with open(out, "w", encoding="utf-8") as f:
    for k in range(6):
        p = idc.get_wide_dword(0x7D2A60 + 4 * k)
        f.write("===== slot %d -> %08X %s\n" % (k, p, idc.get_func_name(p)))
        fn = ida_funcs.get_func(p)
        if fn:
            try:
                f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
            except Exception as e:
                f.write("  decompile failed %s\n" % e)
print("wrote", out)
