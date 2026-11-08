# decompile the call targets of the call instructions listed in research/gfx/callee_in.txt -> research/gfx/callee_out.c
import idc, ida_hexrays
base = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(base + r"\callee_in.txt"):
    l = l.strip()
    if not l: continue
    t = idc.get_operand_value(int(l, 16), 0)
    out.append("//==== site %s -> %X %s" % (l, t, idc.get_func_name(t)))
    try:
        out.append(str(ida_hexrays.decompile(t)))
    except Exception as e:
        out.append("// fail %s" % e)
open(base + r"\callee_out.c", "w").write("\n".join(out) + "\n")
