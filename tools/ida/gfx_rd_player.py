# what VsAIRaceVehicle__24F90()+4 is (RainSurface 0x137090 car index) -> research/gfx/rd_player.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x24F90, 0x761B0, 0x12FD0, 0x5AC40]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\rd_player.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
