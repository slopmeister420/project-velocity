# physics agent: call tree (direct calls + vtable slot calls unresolved) from roots in research/phys_ct_args.txt
import idautils, idc, ida_funcs
roots = [int(x, 16) for x in open(r"D:\projects\project poland rewrite\research\phys_ct_args.txt").read().split()]
seen = {}
order = []
def walk(f, d):
    if f in seen or d > 12:
        return
    seen[f] = d
    order.append((f, d))
    for h in idautils.FuncItems(f):
        if idc.print_insn_mnem(h) != "call":
            continue
        t = idc.get_operand_value(h, 0)
        g = ida_funcs.get_func(t)
        if not g or g.start_ea != t:
            continue
        if t >= 0x3DDD20 and t < 0x500000:
            continue
        walk(t, d + 1)
for r in roots:
    walk(r, 0)
tot = 0
for f, d in order:
    fn = ida_funcs.get_func(f)
    tot += fn.size()
    print("%s%08X %-36s %5d" % ("  " * min(d, 8), f, idc.get_func_name(f), fn.size()))
print("functions", len(order), "bytes", tot)
