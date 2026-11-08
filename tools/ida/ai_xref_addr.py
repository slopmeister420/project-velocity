# data xrefs to the addresses in research/ai_dump/_req.txt, with the instruction (ai agent)
import idautils, ida_funcs, ida_name, idc
out = open(r"D:\projects\project poland rewrite\research\ai_dump\_xref.txt", "w")
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_req.txt"):
    line = line.strip()
    if not line:
        continue
    ea = int(line, 16)
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.write("%X %X %s %s\n" % (ea, x.frm, ida_name.get_name(f.start_ea) if f else "?", idc.GetDisasm(x.frm)))
out.close()
