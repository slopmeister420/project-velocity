# beam lights: callers of SaveFileData 0x106A40 with their pushed arg, users of the global inputs 2/5 (set_vector
# 0x12D090 / set_texture 0x12D130 with id 2 or 5), and the ProjectedLightBeam range function list
import idautils, idc, ida_funcs, ida_ua
out = r"D:/projects/project poland rewrite/research/gfx/beam_xr.txt"
def prev_pushes(ea, n=4):
    r = []
    h = ea
    for _ in range(12):
        h = idc.prev_head(h)
        d = idc.GetDisasm(h)
        if d.startswith("push") or d.startswith("mov     ecx") or d.startswith("mov     edx"):
            r.append("%08X %s" % (h, d))
        if len(r) >= n:
            break
    return r
with open(out, "w", encoding="utf-8") as f:
    for t in (0x106A40, 0x12D090, 0x12D130):
        f.write("== callers of %08X %s\n" % (t, idc.get_func_name(t)))
        for x in idautils.XrefsTo(t):
            fn = ida_funcs.get_func(x.frm)
            pp = prev_pushes(x.frm)
            if t != 0x106A40:
                # only ids 2 / 5
                if not any(("push    2" in p and p.endswith("push    2")) or p.endswith("push    5") for p in pp):
                    continue
            f.write("  %08X %s | %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?", " ; ".join(pp)))
    f.write("== functions 0x161050..0x162B60\n")
    for fe in idautils.Functions(0x161050, 0x162B60):
        f.write("  %08X %s\n" % (fe, idc.get_func_name(fe)))
print("wrote", out)
