# static init of the puddle / mist globals (no function defined there): disassembly -> research/gfx/pud5_init.txt
import idc, idautils
OUT = r"D:\projects\project poland rewrite\research\gfx\pud5_init.txt"
with open(OUT, "w", encoding="utf-8") as f:
    ea = 0x4259F0
    while ea < 0x425BC0:
        f.write("%08X  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
        n = idc.next_head(ea, 0x425BC0)
        if n == idc.BADADDR or n <= ea:
            ea += 1
        else:
            ea = n
print("ok")
