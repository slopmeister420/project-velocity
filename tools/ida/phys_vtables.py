# physics agent: entries of the vtables listed in research/_vtables.txt -> research/_vt_entries.txt
import idc, ida_bytes, ida_funcs
R = r"D:\projects\project poland rewrite\research"
out = []
for line in open(R + r"\_vtables.txt"):
    line = line.strip()
    if not line:
        continue
    vt = int(line, 16)
    k = 0
    while k < 64:
        f = ida_bytes.get_dword(vt + 4 * k)
        fn = ida_funcs.get_func(f)
        if not fn or fn.start_ea != f:
            break
        out.append("%X %d %X %s" % (vt, 4 * k, f, idc.get_func_name(f)))
        k += 1
open(R + r"\_vt_entries.txt", "w").write("\n".join(out))
