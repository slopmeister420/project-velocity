import idc, idautils, ida_funcs
out = []
# writes to GameManager +0x140 (m_bQuitRace)
for s, e, n in [(0x11000, 0x3DDD20, "")]:
    ea = s
    while ea < e:
        d = idc.GetDisasm(ea)
        if "+24h]" in d and d.startswith("mov") and ("Game" in idc.get_func_name(ea)):
            f = ida_funcs.get_func(ea)
            out.append('%x %s | %s' % (ea, d, idc.get_func_name(ea)))
        ea = idc.next_head(ea)
open(r'D:\projects\project poland rewrite\research\core_q.txt', 'w').write('\n'.join(out))
