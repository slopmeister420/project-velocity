import idc, idautils, ida_funcs
# disassembly of the functions listed in research/core_chase_dis_in.txt (hex, one per line) to research/core_chase_dis.txt
base = r"D:\projects\project poland rewrite\research"
out = []
for l in open(base + r"\core_chase_dis_in.txt"):
    s = l.strip()
    if not s:
        continue
    parts = s.split()
    a = int(parts[0], 16)
    if len(parts) > 1:
        S, E = a, int(parts[1], 16)
    else:
        f = ida_funcs.get_func(a)
        S, E = f.start_ea, f.end_ea
    out.append("==== %X %s" % (S, idc.get_func_name(S)))
    for ea in idautils.Heads(S, E):
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
open(base + r"\core_chase_dis.txt", "w").write("\n".join(out))
print("ok")
