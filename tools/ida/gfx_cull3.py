# disassembly around the CULLMODE (0x93) render-state calls
import idc
out = "D:/projects/project poland rewrite/research/gfx/cull_dis.txt"
with open(out, "w", encoding="utf-8") as f:
    for c in (0x2A448D, 0x2A65D1, 0x2A83D4, 0x2AA97D):
        f.write("==== %08X\n" % c)
        h = c
        for _ in range(14):
            h = idc.prev_head(h)
        for _ in range(22):
            f.write("%08X %s\n" % (h, idc.generate_disasm_line(h, 0)))
            h = idc.next_head(h)
print("wrote", out)
