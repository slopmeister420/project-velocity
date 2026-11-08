import ida_hexrays, idc
# lines around each VsCameraManager__60380 (GameBridge cut flag set) call -> research/core_cut_ctx.txt
out = []
for ea in (0x62C10, 0x62D90, 0x64090, 0xB77F0, 0xBC0C0, 0xBC2C0, 0xC4460, 0x1781C0, 0x1783C0, 0x174A70):
    s = str(ida_hexrays.decompile(ea)).split("\n")
    for i, ln in enumerate(s):
        if "60380" in ln or "176970" in ln:
            out.append("==== %X %s line %d" % (ea, idc.get_func_name(ea), i))
            out.extend(x[:140] for x in s[max(0, i - 8):i + 3])
open(r"D:/projects/project poland rewrite/research/core_cut_ctx.txt", "w").write("\n".join(out))
