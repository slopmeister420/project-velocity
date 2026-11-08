import idc, ida_funcs, idautils
root = r"D:\projects\project poland rewrite\research"
addrs = []
for l in open(root + r"\ai_coverage.txt"):
    if l.startswith("0x"):
        addrs.append(int(l.split("\t")[0], 16))
out = []
for a in addrs:
    n = idc.get_func_name(a) or ""
    callers = len(set(ida_funcs.get_func(x).start_ea for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)))
    drefs = len(list(idautils.DataRefsTo(a)))
    out.append("%08X\t%s\tcallers=%d\tdrefs=%d" % (a, n, callers, drefs))
open(root + r"\ai_dump\cov_names.tsv", "w").write("\n".join(out) + "\n")
print(len(out))
