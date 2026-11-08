# physics agent stage 4b: does each listed function read edx before writing it? (fastcall vs thiscall)
import idc, idautils, ida_funcs
out = []
for a in [0x1A89F0, 0x1AAED0, 0x1E4C30, 0x1DEBB0, 0x1E4710, 0x1E3A60, 0x1E1F50, 0x384C70, 0x1E5270, 0x379660,
          0x3847C0, 0x379050]:
    f = ida_funcs.get_func(a)
    ea = a
    verdict = "no edx"
    n = 0
    while ea < f.end_ea and n < 400:
        d = idc.generate_disasm_line(ea, 0)
        ops = d.split(None, 1)
        if len(ops) > 1 and "edx" in ops[1] or (len(ops) > 1 and " dl" in ops[1]):
            dst = ops[1].split(",")[0]
            if ops[0] in ("mov", "lea", "movzx", "pop") and "edx" in dst and "[" not in dst:
                verdict = "edx written first at %X: %s" % (ea, d)
            else:
                verdict = "edx READ at %X: %s" % (ea, d)
            break
        if ops[0] == "call":
            verdict = "call before edx at %X" % ea
            break
        ea = idc.next_head(ea, f.end_ea)
        n += 1
    out.append("%X %s | %s" % (a, idc.get_type(a), verdict))
    # stack arg cleanup
    ea = f.end_ea
    out.append("   ret: %s" % idc.generate_disasm_line(idc.prev_head(f.end_ea, a), 0))
open(r"D:\projects\project poland rewrite\research\phys_s4b_edx.txt", "w").write("\n".join(out))
