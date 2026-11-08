# physics agent: list config reader calls in CConfig load functions, in ctree order, with full string args
import ida_hexrays, ida_lines, idc, ida_bytes
READERS = {0x1B6570: "INT", 0x1B6710: "REAL", 0x1B68B0: "STR", 0x1B6A20: "EXPR"}
OUT = r"D:\projects\project poland rewrite\research\phys_dump\cfgcalls.txt"
res = []

class V(ida_hexrays.ctree_visitor_t):
    def __init__(self):
        ida_hexrays.ctree_visitor_t.__init__(self, ida_hexrays.CV_PARENTS)
    def visit_insn(self, i):
        if i.op in (ida_hexrays.cit_for, ida_hexrays.cit_while, ida_hexrays.cit_do):
            res.append("   ## loop @%X" % i.ea)
        return 0
    def visit_expr(self, e):
        if e.op == ida_hexrays.cot_call and e.x.op == ida_hexrays.cot_obj and e.x.obj_ea in READERS:
            args = []
            for a in e.a:
                x = a
                while x.op == ida_hexrays.cot_cast:
                    x = x.x
                if x.op == ida_hexrays.cot_obj and ida_bytes.is_strlit(ida_bytes.get_flags(x.obj_ea)):
                    args.append('"%s"' % idc.get_strlit_contents(x.obj_ea).decode("latin1"))
                else:
                    args.append(ida_lines.tag_remove(a.print1(None)))
            res.append("%s @%X %s" % (READERS[e.x.obj_ea], e.ea, " | ".join(args)))
        return 0

for fea in (0x18F870, 0x19A220):
    res.append("#### %X" % fea)
    cf = ida_hexrays.decompile(fea)
    V().apply_to(cf.body, None)
open(OUT, "w").write("\n".join(res))
print(len(res))
