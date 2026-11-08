# physics agent: disassemble the ranges in research/damage_dump/dis.txt ("start end" hex per line) -> dis_out.txt
import os, idc, ida_ua

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
with open(os.path.join(ROOT, "dis_out.txt"), "w") as o:
    for line in open(os.path.join(ROOT, "dis.txt")):
        p = line.split()
        if len(p) < 2:
            continue
        a, e = int(p[0], 16), int(p[1], 16)
        while a < e:
            o.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
            n = ida_ua.decode_insn(ida_ua.insn_t(), a)
            a += n if n > 0 else 1
print("ok")
