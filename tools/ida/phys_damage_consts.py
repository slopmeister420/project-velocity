# physics agent: CarDamage constants + default bone table + helper decompiles -> research/damage_dump/consts.txt
import os, struct, idc, idautils, ida_bytes, ida_hexrays, ida_funcs, ida_name

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
o = open(os.path.join(ROOT, "consts.txt"), "w", encoding="utf-8")
for a in [0x685264, 0x685268, 0x68526C, 0x685270, 0x685274, 0x685278, 0x68527C, 0x685280, 0x685284, 0x685288,
          0x6852BC, 0x6852C0, 0x68528C, 0x685290, 0x685294, 0x685298, 0x68529C, 0x6852A0]:
    w = ida_bytes.get_dword(a)
    o.write("%08X %08X %r\n" % (a, w, struct.unpack("<f", struct.pack("<I", w))[0]))
o.write("bone table 6D63C8:\n")
for i in range(9):
    b = [ida_bytes.get_dword(0x6D63C8 + 40 * i + 4 * k) for k in range(10)]
    o.write("%d %s\n" % (i, " ".join("%08X" % x for x in b)))
o.write("xrefs 6D63C8:\n")
for x in idautils.XrefsTo(0x6D63C8):
    f = ida_funcs.get_func(x.frm)
    o.write("  %08X %s\n" % (x.frm, ida_name.get_name(f.start_ea) if f else "?"))
for a in [0x76490, 0x6FE50, 0x763F0, 0x76560, 0x764A0, 0x76270, 0x761B0, 0x762F0, 0x6F5A0, 0x6F120, 0x75AB0]:
    try:
        o.write("// ==== %08X\n%s\n" % (a, ida_hexrays.decompile(a)))
    except Exception as ex:
        o.write("// fail %08X %s\n" % (a, ex))
o.close()
print("ok")
