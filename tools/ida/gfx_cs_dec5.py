# SetShaderStage 0x2A4960 -> research/gfx/cs_dec5.c ; CompileStage 0x2A3320 -> cs_dec6.c
import ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
open(B + r"\cs_dec5.c", "w", encoding="utf-8").write(str(ida_hexrays.decompile(0x2A4960)))
open(B + r"\cs_dec6.c", "w", encoding="utf-8").write(str(ida_hexrays.decompile(0x2A3320)))
print("ok")
