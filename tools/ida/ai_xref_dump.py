# xrefs to an address listed in research/ai_dump/_xa.txt, with the referencing instruction (ai agent)
import idautils, ida_funcs, ida_name, idc
OUT = r"D:\projects\project poland rewrite\research\ai_dump\_xa_out.txt"
with open(OUT, "w") as o:
    for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_xa.txt"):
        line = line.strip()
        if not line:
            continue
        ea = int(line, 16)
        o.write("== %08X\n" % ea)
        for x in idautils.XrefsTo(ea):
            f = ida_funcs.get_func(x.frm)
            o.write("  %08X %s: %s\n" % (x.frm, ida_name.get_name(f.start_ea) if f else "?", idc.GetDisasm(x.frm)))
            # the next few instructions
            n = x.frm
            for _ in range(4):
                n = idc.next_head(n)
                o.write("      %08X %s\n" % (n, idc.GetDisasm(n)))
print("ok")
