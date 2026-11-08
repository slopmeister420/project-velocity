# frontend agent: disassemble the instructions before each call inside a function (fe_peek_args.txt: func callee_substring)
import idautils, idc, ida_funcs
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
f = ida_funcs.get_func(int(a[0], 16))
for h in idautils.Heads(f.start_ea, f.end_ea):
    d = idc.GetDisasm(h)
    if d.startswith("call") and a[1] in d:
        p = h
        lines = []
        for _ in range(8):
            p = idc.prev_head(p)
            lines.append("  %08X  %s" % (p, idc.GetDisasm(p)))
        print("\n".join(reversed(lines)))
        print("  %08X  %s\n" % (h, d))
