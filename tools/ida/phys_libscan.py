# physics agent: find functions referencing strings in a data window (locate the unattributed physics library)
import idautils, ida_funcs, ida_name, idc
LO, HI = 0x675000, 0x676A50
hits = {}
for s in idautils.Strings():
    if LO <= s.ea < HI:
        for x in idautils.XrefsTo(s.ea):
            f = ida_funcs.get_func(x.frm)
            if f:
                hits.setdefault(f.start_ea, []).append(str(s)[:50])
for f in sorted(hits):
    print("%08X %-30s %s" % (f, ida_name.get_name(f), " | ".join(sorted(set(hits[f])))[:200]))
