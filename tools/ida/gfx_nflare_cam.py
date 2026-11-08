# VsBaseCamera.cpp functions touching +8 (the camera matrix) -> research/gfx/nflare_cam.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
ea = 0x5D0B0
while ea < 0x5ECC0:
    f = ida_funcs.get_func(ea)
    if not f:
        ea = idc.next_head(ea)
        continue
    try:
        s = str(ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        s = "// fail %s" % ex
    if ("+ 8" in s or "this + 2)" in s) and ("88" in s or "D3DX" in s or "sub_48" in s):
        out.append("//==== %08X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
        out.append(s)
    ea = f.end_ea
open(B + r"\nflare_cam.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
