import idaapi, idc, idautils, ida_hexrays, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/core_lights_rate.txt", "w")
def dec(ea):
    try:
        f = ida_funcs.get_func(ea)
        out.write("==== %X %s\n" % (f.start_ea, idc.get_func_name(f.start_ea)))
        out.write(str(ida_hexrays.decompile(f.start_ea)) + "\n")
    except Exception as e:
        out.write("fail %X %s\n" % (ea, e))
dec(0x112A50)
# strings about refresh / PAL / presentation interval
for s in idautils.Strings():
    t = str(s)
    tl = t.lower()
    if any(k in tl for k in ("pal", "ntsc", "50hz", "60hz", "refresh", "presentationinterval", "interval", "vblank", "fullscreen_refresh")) and len(t) < 120:
        refs = [r.frm for r in idautils.XrefsTo(s.ea)]
        out.write("STR %X %r refs %s\n" % (s.ea, t, " ".join("%X" % r for r in refs[:6])))
# XGetVideoFlags / D3D present
for name in ("XGetVideoFlags", "XGetVideoStandard", "D3DDevice_SetFlickerFilter", "D3DDevice_Swap", "D3DDevice_Present", "D3DDevice_BlockUntilVerticalBlank", "D3DDevice_SetVerticalBlankCallback", "Direct3D_CreateDevice", "D3DDevice_PersistDisplay"):
    for ea, n in idautils.Names():
        if name.lower() in n.lower():
            refs = [r.frm for r in idautils.XrefsTo(ea)]
            out.write("NAME %X %s refs %s\n" % (ea, n, " ".join("%X" % r for r in refs[:10])))
out.close()
