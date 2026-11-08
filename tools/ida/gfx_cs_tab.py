import idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for base in (0x65CE30, 0x65CE50):
    out.append("%X: %s" % (base, " ".join("%X" % idc.get_wide_dword(base + 4 * i) for i in range(8))))
open(B + r"\cs_tab.txt", "w").write("\n".join(out))
print("ok")
