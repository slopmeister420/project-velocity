# frontend agent: callers of the given functions with 10 instructions before each call
import idautils, idc
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    ea = int(a, 16)
    for x in idautils.CodeRefsTo(ea, 0):
        print("== %08X %s" % (x.frm if hasattr(x,'frm') else x, idc.get_func_name(x)))
        p = x; l = []
        for _ in range(10):
            p = idc.prev_head(p); l.append("  %08X  %s" % (p, idc.GetDisasm(p)))
        print("\n".join(reversed(l)))
