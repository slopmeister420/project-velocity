import idc, ida_funcs, idautils, ida_bytes
# audio scope: every function named *AUDIO_* (visos Audio layer), plus src/Audio + TimelineSound/GlobalAudio regions
root = r"D:\projects\project poland rewrite\research"
regs = [(0x49280, 0x56d90), (0x17b480, 0x17bd70), (0x17ec30, 0x17f380)]
addrs = {}
for a in idautils.Functions():
    n = idc.get_func_name(a) or ""
    if "AUDIO_" in n and a < 0x3d0000:
        addrs[a] = "visos"
    else:
        for s, e in regs:
            if s <= a < e:
                addrs[a] = "game"
out = []
for a in sorted(addrs):
    n = idc.get_func_name(a) or ""
    f = ida_funcs.get_func(a)
    callers = len(set(ida_funcs.get_func(x).start_ea for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)))
    drefs = len(list(idautils.DataRefsTo(a)))
    flags = set()
    if f:
        for h in idautils.FuncItems(a):
            for d in idautils.DataRefsFrom(h):
                s = idc.get_strlit_contents(d, -1, 0)
                if s:
                    low = s.decode("latin1", "replace").lower()
                    if "debug" in low or "tweak" in low:
                        flags.add("dbgstr")
                    if "assert" in low:
                        flags.add("assert")
    out.append("%08X\t%s\t%d\t%d\t%s\t%s" % (a, n, callers, drefs, ",".join(sorted(flags)), addrs[a]))
open(root + r"\audio_names.tsv", "w").write("\n".join(out) + "\n")
print(len(out))
