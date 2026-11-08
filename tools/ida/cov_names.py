import idc, ida_funcs, idautils, ida_bytes
root = r"D:\projects\project poland rewrite\research"
addrs = []
for l in open(root + r"\core_coverage.txt"):
    if l.startswith("0x"):
        addrs.append(int(l.split("\t")[0], 16))
out = []
for a in addrs:
    f = ida_funcs.get_func(a)
    n = idc.get_func_name(a) or ""
    callers = len(set(ida_funcs.get_func(x).start_ea for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)))
    drefs = len(list(idautils.DataRefsTo(a)))
    flags = set()
    if f:
        for h in idautils.FuncItems(a):
            for d in idautils.DataRefsFrom(h):
                s = idc.get_strlit_contents(d, -1, 0)
                if s:
                    s = s.decode("latin1", "replace")
                    low = s.lower()
                    if "debug" in low or "tweak" in low:
                        flags.add("dbgstr")
                    if "assert" in low:
                        flags.add("assert")
                elif ida_bytes.is_loaded(d) and idc.is_code(idc.get_full_flags(idc.get_wide_dword(d))) \
                        and ida_funcs.get_func(idc.get_wide_dword(d)) and idc.get_wide_dword(d) != a \
                        and ida_funcs.get_func(idc.get_wide_dword(d)).start_ea == idc.get_wide_dword(d):
                    flags.add("vtbl")
            for c in idautils.CodeRefsFrom(h, 0):
                cn = idc.get_func_name(c) or ""
                if "MEM_Free" in cn or "delete" in cn:
                    flags.add("free")
    out.append("%08X\t%s\t%d\t%d\t%s" % (a, n, callers, drefs, ",".join(sorted(flags))))
open(root + r"\core_names.tsv", "w").write("\n".join(out) + "\n")
print(len(out))
