import idautils, idc, ida_segment, ida_funcs, ida_nalt, re, collections
out=[]
for s in idautils.Segments():
    seg=ida_segment.getseg(s)
    out.append("seg %-10s %08x-%08x" % (ida_segment.get_segm_name(seg), seg.start_ea, seg.end_ea))
funcs=list(idautils.Functions())
named=[f for f in funcs if not idc.get_func_name(f).startswith("sub_")]
out.append("funcs %d named %d" % (len(funcs), len(named)))
srcs=collections.Counter(); classes=set()
for s in idautils.Strings():
    t=str(s)
    m=re.search(r'[\/]src[\/](.+\.(cpp|h|c|inl))$', t, re.I)
    if m: srcs[m.group(1).lower()]+=1
    if t.startswith(".?AV") or t.startswith(".?AU"): classes.add(t)
out.append("src files %d rtti %d" % (len(srcs), len(classes)))
open(r"D:\projects\project poland rewrite\research\srcfiles.txt","w").write("\n".join(sorted(srcs)))
open(r"D:\projects\project poland rewrite\research\rtti.txt","w").write("\n".join(sorted(classes)))
print("\n".join(out))
print("named sample:", [idc.get_func_name(f) for f in named[:60]])
