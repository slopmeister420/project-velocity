# CSpoke__47840 and its call site in TrackFX 0x14BE40 -> research/gfx/hb_dec3.txt
import ida_hexrays, idc, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = [str(ida_hexrays.decompile(0x47840))]
f = ida_funcs.get_func(0x14BE40)
ea = f.start_ea
while ea < f.end_ea:
    l = idc.generate_disasm_line(ea, 0)
    if "47840" in l:
        p = ea
        for _ in range(8):
            p = idc.prev_head(p)
        q = p
        while q <= ea:
            out.append("%X %s" % (q, idc.generate_disasm_line(q, 0)))
            q = idc.next_head(q)
    ea = idc.next_head(ea)
open(B + r"\hb_dec3.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
