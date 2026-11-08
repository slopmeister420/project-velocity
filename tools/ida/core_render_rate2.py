import idaapi, idc, idautils, ida_hexrays, ida_funcs, ida_ua
out = open(r"D:/projects/project poland rewrite/research/core_render_rate2.txt", "w")
def dec(ea, tag=""):
    f = ida_funcs.get_func(ea)
    if not f:
        out.write("nofunc %X\n" % ea); return
    try:
        out.write("==== %s %X %s\n" % (tag, f.start_ea, idc.get_func_name(f.start_ea)))
        out.write(str(ida_hexrays.decompile(f.start_ea)) + "\n")
    except Exception as e:
        out.write("fail %X %s\n" % (ea, e))
def fstart(ea):
    f = ida_funcs.get_func(ea)
    return f.start_ea if f else None
# walk callers of the CreateDevice validator up into game code
seen = set()
frontier = [fstart(0x44B189), fstart(0x452AF3)]
for depth in range(5):
    nxt = []
    for fe in frontier:
        if fe is None or fe in seen: continue
        seen.add(fe)
        out.write("CHAIN d%d %X %s\n" % (depth, fe, idc.get_func_name(fe)))
        if 0x11000 <= fe < 0x3DDD20:
            dec(fe, "game")
            continue
        for r in idautils.XrefsTo(fe):
            nxt.append(fstart(r.frm))
    frontier = nxt
# timer ctor callers
for r in idautils.XrefsTo(0x572A0):
    dec(r.frm, "timer-ctor-caller")
# writes of [x+0Ch] near references to the timer global: find globals passed to 57580
for r in idautils.XrefsTo(0x57580):
    dec(r.frm, "timer-tick-caller")
out.close()
