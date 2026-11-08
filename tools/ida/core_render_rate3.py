import idaapi, idc, idautils, ida_hexrays, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/core_render_rate3.txt", "w")
def dec(ea, tag=""):
    f = ida_funcs.get_func(ea)
    if not f:
        out.write("nofunc %X\n" % ea); return
    try:
        out.write("==== %s %X %s\n" % (tag, f.start_ea, idc.get_func_name(f.start_ea)))
        out.write(str(ida_hexrays.decompile(f.start_ea)) + "\n")
    except Exception as e:
        out.write("fail %X %s\n" % (ea, e))
for r in idautils.XrefsTo(0x2AAF60):
    dec(r.frm, "createdevice-caller")
# every function touching the timer global: print lines mentioning +12 / 0Ch
fs = set()
for r in idautils.XrefsTo(0x74A430):
    f = ida_funcs.get_func(r.frm)
    if f: fs.add(f.start_ea)
for fe in sorted(fs):
    out.write("TIMERREF %X %s\n" % (fe, idc.get_func_name(fe)))
    for h in idautils.FuncItems(fe):
        d = idc.GetDisasm(h)
        if "+0Ch]" in d and "mov" in d:
            out.write("   %X %s\n" % (h, d))
# all strings with Hz / pal / 50 / 60 in the game range
for s in idautils.Strings():
    t = str(s)
    tl = t.lower()
    if ("hz" in tl or "pal" in tl or "ntsc" in tl or "vblank" in tl or "vsync" in tl) and len(t) < 100:
        refs = [r.frm for r in idautils.XrefsTo(s.ea) if 0x11000 <= r.frm < 0x3DDD20]
        if refs:
            out.write("STR %X %r refs %s\n" % (s.ea, t, " ".join("%X" % x for x in refs[:6])))
out.close()
