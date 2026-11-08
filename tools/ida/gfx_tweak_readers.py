import idc, ida_hexrays
# lines of each reader function that mention a tweak variable (+-2 lines) -> research/gfx/tweak_readers.txt
PAIRS = [(0x1134A0, "7065A8"), (0x116F60, "686610"), (0x114A90, "686600"), (0x118A40, "7065BA"),
         (0x111920, "7065B"), (0x13F770, "6870BC"), (0x1618D0, "7486AC"), (0x147DC0, "73694A"), (0xB9010, "73694A")]
o = open(r"D:\projects\project poland rewrite\research\gfx\tweak_readers.txt", "w")
for f, v in PAIRS:
    o.write("== %x %s %s\n" % (f, idc.get_func_name(f), v))
    try:
        ls = str(ida_hexrays.decompile(f)).split("\n")
    except Exception as ex:
        o.write("ERR %s\n" % ex); continue
    for i, l in enumerate(ls):
        if v in l:
            for j in range(max(0, i - 2), min(len(ls), i + 4)):
                o.write("%5d %s\n" % (j, ls[j].rstrip()[:150]))
            o.write("  --\n")
o.close()
print("ok")
