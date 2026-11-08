# physics agent: find instructions whose operand text contains a pattern within ranges
# args file research/phys_findop_args.txt: line1 = pattern (regex), following lines "start end"
import idautils, idc, re, ida_funcs
lines = open(r"D:\projects\project poland rewrite\research\phys_findop_args.txt").read().split("\n")
pat = re.compile(lines[0].strip())
for l in lines[1:]:
    p = l.split()
    if len(p) < 2:
        continue
    s, e = int(p[0], 16), int(p[1], 16)
    for h in idautils.Heads(s, e):
        if not idc.is_code(idc.get_full_flags(h)):
            continue
        t = idc.generate_disasm_line(h, 0)
        if pat.search(t):
            print("%08X %-28s %s" % (h, idc.get_func_name(h), t))
