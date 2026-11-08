import idc, idautils, ida_funcs
root = r"D:\projects\project poland rewrite\research"
out = []
for a in [0x9EF30, 0x5E220, 0xA68D0, 0x1813C0, 0xC1480, 0x175870, 0x9DA10, 0x9AED0, 0xBEBB0, 0xA0B40, 0x16F860, 0x171B90, 0xCB440, 0xCA4C0, 0x171F20, 0xA5FA0, 0x9A1F0]:
    ss = []
    for h in idautils.FuncItems(a):
        for d in idautils.DataRefsFrom(h):
            s = idc.get_strlit_contents(d, -1, 0)
            if s and len(s) > 3 and b"\\" not in s[:3]:
                t = s.decode("latin1")[:40]
                if t not in ss and "src" not in t:
                    ss.append(t)
    callers = [idc.get_func_name(ida_funcs.get_func(x).start_ea) for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)][:2]
    out.append("%X|%s|%s" % (a, ss[:4], callers))
open(root + r"\core_hints.txt", "w").write("\n".join(out))
