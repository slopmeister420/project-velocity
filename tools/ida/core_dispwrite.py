import idc, idautils, ida_ua
# writes ("mov [reg+disp], x") with disp in research/core_dispwrite.in (first line: start end, then disps) -> .txt
p = open(r"D:/projects/project poland rewrite/research/core_dispwrite.in").read().split()
s, e = int(p[0], 16), int(p[1], 16)
want = set(int(x, 16) for x in p[2:])
out = []
for f in idautils.Functions(s, e):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        if idc.print_insn_mnem(h) != "mov":
            continue
        op = insn.ops[0]
        if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) in want:
            out.append("%08X %s | %s" % (h, idc.get_func_name(h), idc.GetDisasm(h)))
open(r"D:/projects/project poland rewrite/research/core_dispwrite.txt", "w").write("\n".join(out))
