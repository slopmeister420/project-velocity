import ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
open(B + r"\cs_dec9.c", "w", encoding="utf-8").write(str(ida_hexrays.decompile(0x290F20)))
print("ok")
