# physics agent: for each call to the targets below, print the call and the next 3 instructions (how the caller
# tests the return: `test eax, eax` = 32-bit, `or eax, edx` = 64-bit) -> research/phys_dump/callsite_test.txt
import idc, idautils, ida_ua, ida_funcs

TARGETS = [0x37B860, 0x394EA0, 0x38CF90, 0x39F220, 0x3813B0, 0x37B520]
OUT = r"D:\projects\project poland rewrite\research\phys_dump\callsite_test.txt"
with open(OUT, "w") as o:
    for t in TARGETS:
        o.write("== %06X %s\n" % (t, idc.get_func_name(t)))
        for x in idautils.CodeRefsTo(t, 0):
            f = ida_funcs.get_func(x)
            a = idc.next_head(x)
            seq = []
            for _ in range(3):
                seq.append(idc.generate_disasm_line(a, 0))
                a = idc.next_head(a)
            o.write("  %06X in %s: %s\n" % (x, idc.get_func_name(f.start_ea) if f else "?", " | ".join(seq)))
print("ok")
