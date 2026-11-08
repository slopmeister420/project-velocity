# attribute every .text function to a source file using __FILE__ xrefs + obj contiguity
import idautils, idc, ida_funcs, ida_segment, re, collections, json
srcre = re.compile(r'\.(cpp|c|h|inl)$', re.I)
paths = {s.ea: str(s) for s in idautils.Strings() if srcre.search(str(s))}
text = ida_segment.get_segm_by_name(".text")
funcs = [f for f in idautils.Functions(text.start_ea, text.end_ea)]
direct = {}
for ea, p in paths.items():
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        if f and text.start_ea <= f.start_ea < text.end_ea:
            direct.setdefault(f.start_ea, collections.Counter())[p] += 1
def norm(p):
    p = p.replace("/", "\\")
    return re.sub(r'^(\.\.[\\/])+|^\.[\\/]', '', p)
# prefer .cpp/.c over headers (inlined header asserts)
own = {}
for f, c in direct.items():
    cpp = [p for p in c if re.search(r'\.(cpp|c)$', p, re.I)]
    if cpp:
        own[f] = norm(max(cpp, key=lambda p: c[p]))
# fill gaps: a function between two functions of the same file inherits it
assigned = {}
prev = None
seq = [(f, own.get(f)) for f in funcs]
last_idx = {}
for i, (f, o) in enumerate(seq):
    if o:
        if o in last_idx and i - last_idx[o] > 1:
            # only fill if nothing else claimed in between
            between = [seq[j][1] for j in range(last_idx[o] + 1, i)]
            if all(b is None for b in between):
                for j in range(last_idx[o] + 1, i):
                    assigned[seq[j][0]] = o
        last_idx[o] = i
        assigned[f] = o
rows = []
for f in funcs:
    rows.append("%08x\t%d\t%s\t%s" % (f, ida_funcs.get_func(f).size(), assigned.get(f, ""), "D" if f in own else ""))
open(r"D:\projects\project poland rewrite\research\funcmap.tsv", "w").write("\n".join(rows))
cnt = collections.Counter(v for v in assigned.values())
print("text funcs", len(funcs), "direct", len(own), "assigned", len(assigned), "files", len(cnt))
