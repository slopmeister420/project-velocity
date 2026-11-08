import os, idautils, idc, ida_funcs, ida_bytes, idaapi
# list functions in ranges: addr size name ncallers strings-referenced
RANGES = [(0x46b30, 0x56d90), (0x2b8820, 0x2bf2d0), (0x3134b0, 0x315a40),
          (0x340400, 0x343290), (0x354f80, 0x3c6cf0)]
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "research", "audio", "audio_list.txt")
with open(OUT, "w", encoding="utf-8") as fo:
    for a, b in RANGES:
        fo.write("=== %x-%x\n" % (a, b))
        for f in idautils.Functions(a, b):
            fn = ida_funcs.get_func(f)
            ncall = len(list(idautils.CodeRefsTo(f, 0)))
            strs = []
            for h in idautils.FuncItems(f):
                for d in idautils.DataRefsFrom(h):
                    if ida_bytes.is_strlit(ida_bytes.get_flags(d)):
                        s = idc.get_strlit_contents(d)
                        if s:
                            strs.append(s.decode("latin1")[:40])
            callees = set()
            for h in idautils.FuncItems(f):
                for c in idautils.CodeRefsFrom(h, 0):
                    g = ida_funcs.get_func(c)
                    if g and g.start_ea != f:
                        callees.add(idc.get_func_name(g.start_ea))
            fo.write("%06x %5d %-40s c=%d | %s | -> %s\n" % (f, fn.end_ea - f, idc.get_func_name(f), ncall,
                     " ; ".join(strs)[:200].replace("\n", " "), ",".join(sorted(callees))[:300]))
print("ok")
