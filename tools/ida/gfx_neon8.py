# FE3DVehicles 0x25C890 call of CarRender 0x1100B0 (a6 = car data +1816) -> research/gfx/neon_fe.c
import ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
txt = str(ida_hexrays.decompile(0x25C890))
lines = txt.split("\n")
out = []
for i, l in enumerate(lines):
    if "CarRender__1100B0" in l:
        out.extend(lines[max(0, i - 25): i + 8])
open(B + r"\neon_fe.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
