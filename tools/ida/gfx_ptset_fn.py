# what lives in 0x113AD3..0x114A90 (unk_7181C4 / flt_706534 readers): function chunks, code refs into it
import idautils, idc, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/ptset_fn.txt"
with open(out, "w", encoding="utf-8") as f:
    ea = 0x113AD0
    while ea < 0x114A90:
        fn = ida_funcs.get_func(ea)
        crefs = list(idautils.CodeRefsTo(ea, 0))
        if crefs and not any(0x113AD0 <= c < 0x114A90 for c in crefs):
            f.write("%08X fn=%s external crefs %s\n" % (ea, idc.get_func_name(fn.start_ea) if fn else None, " ".join("%08X(%s)" % (c, idc.get_func_name(c)) for c in crefs)))
        nxt = idc.next_head(ea, 0x114A90)
        if nxt == idc.BADADDR:
            break
        ea = nxt
    for a in (0x113B00, 0x113D9C, 0x114285, 0x114747):
        fn = ida_funcs.get_func(a)
        f.write("%08X in fn %s chunk %s\n" % (a, hex(fn.start_ea) if fn else None, idc.get_fchunk_attr(a, idc.FUNCATTR_START)))
    for h in idautils.Heads(0x113AC0, 0x113B30):
        f.write("%08X %s\n" % (h, idc.GetDisasm(h)))
print("ok")
