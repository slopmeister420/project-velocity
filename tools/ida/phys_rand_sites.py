import idautils, idc, ida_funcs
out = []
def path(root, target):
    par = {root: None}
    q = [root]
    while q:
        f = q.pop(0)
        if f == target:
            p = []
            while f is not None:
                p.append("%08X %s" % (f, idc.get_func_name(f)))
                f = par[f]
            return " <- ".join(p)
        if not ida_funcs.get_func(f):
            continue
        for ea in idautils.FuncItems(f):
            for x in idautils.CodeRefsFrom(ea, 0):
                g = ida_funcs.get_func(x)
                if g and g.start_ea == x and x not in par and (x < 0x3DDD20 or x == target):
                    par[x] = f
                    q.append(x)
    return "none"
for r, t in ((0x98940, 0xCD430), (0xA5A40, 0xCD430), (0x1E1F80, 0x2BB910), (0x1E1F80, 0x1E1A20)):
    out.append("%08X -> %08X: %s" % (r, t, path(r, t)))
with open(r"D:\projects\project poland rewrite\research\phys_rand_sites.txt", "w") as fh:
    fh.write("\n".join(out))
