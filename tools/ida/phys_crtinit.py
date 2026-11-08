# physics agent: static initialisers (the 0x43xxxx init stubs) that call into the physics/FastCar
# ranges: each call with the instructions before it, to research/_crtinit.txt
import idc, idautils, ida_segment
R = r"D:\projects\project poland rewrite\research"
RANGES = [(0x182530, 0x1E6130), (0x355B60, 0x3C63D0)]
out = []
ea = 0x428000
end = 0x440000
while ea < end:
    if idc.print_insn_mnem(ea) == "call":
        t = idc.get_operand_value(ea, 0)
        if any(a <= t < b for a, b in RANGES):
            ctx = []
            h = ea
            for _ in range(6):
                h = idc.prev_head(h, ea - 64)
                if h == idc.BADADDR:
                    break
                ctx.insert(0, "  %X %s" % (h, idc.generate_disasm_line(h, 0)))
                if idc.print_insn_mnem(h) in ("retn", "push") and idc.print_operand(h, 0) == "ebp":
                    break
            out.append("%X call %X %s\n%s" % (ea, t, idc.get_func_name(t), "\n".join(ctx)))
    n = idc.next_head(ea, end)
    if n == idc.BADADDR:
        break
    ea = n
open(R + r"\_crtinit.txt", "w").write("\n".join(out))
