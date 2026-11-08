import idaapi, idc, idautils, ida_hexrays, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/core_render_rate.txt", "w")
def dec(ea, tag=""):
    f = ida_funcs.get_func(ea)
    if not f:
        out.write("nofunc %X\n" % ea); return
    try:
        out.write("==== %s %X %s\n" % (tag, f.start_ea, idc.get_func_name(f.start_ea)))
        out.write(str(ida_hexrays.decompile(f.start_ea)) + "\n")
    except Exception as e:
        out.write("fail %X %s\n" % (ea, e))
# VsFrameTimer funcs
for ea in idautils.Functions(0x572A0, 0x57D00):
    dec(ea, "timer")
# platform names of interest and their game-code callers
keys = ("xgetvideo", "createdevice", "presentationinterval", "setflicker", "blockuntilverticalblank",
        "setverticalblankcallback", "getdisplaymode", "d3ddevice_reset", "xgetavpack", "getrefresh", "swap")
seen = set()
for ea, n in idautils.Names():
    nl = n.lower()
    if any(k in nl for k in keys):
        refs = [r.frm for r in idautils.XrefsTo(ea)]
        out.write("NAME %X %s refs %s\n" % (ea, n, " ".join("%X" % r for r in refs[:12])))
        for r in refs:
            if 0x11000 <= r < 0x3DDD20:
                f = ida_funcs.get_func(r)
                if f and f.start_ea not in seen:
                    seen.add(f.start_ea)
                    dec(r, "caller-of-" + n)
out.close()
