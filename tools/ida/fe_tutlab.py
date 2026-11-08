# frontend agent: loading screen tutorial labels (0x00241670..0x00241940) -> research/fe_tutlab.txt
import idc, ida_hexrays, ida_bytes, idautils, struct
out = open(r"D:\projects\project poland rewrite\research\fe_tutlab.txt", "w", encoding="utf-8")
for ea in [0x1E7360, 0x1E6F80, 0x293670, 0x293590, 0x2B16B0]:
    f = idc.get_func_attr(ea, idc.FUNCATTR_START)
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
out.write("// off_6C1850\n")
for i in range(12):
    p = ida_bytes.get_dword(0x6C1850 + 4 * i)
    out.write("%d %08X %r\n" % (i, p, idc.get_strlit_contents(p, -1, 0)))
out.write("// dword_76A518 raw (bss?) seg=%s\n" % idc.get_segm_name(0x76A518))
for i in range(60):
    v = ida_bytes.get_dword(0x76A518 + 4 * i)
    out.write("%d %08X %e\n" % (i, v, struct.unpack("<f", struct.pack("<I", v))[0]))
out.write("// xrefs 76A518..76A608\n")
seen = set()
for a in range(0x76A510, 0x76A610, 4):
    for x in idautils.XrefsTo(a):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("%08X -> %08X in %08X %s\n" % (x.frm, a, f, idc.get_func_name(f)))
        seen.add(f)
for f in sorted(seen):
    if f == 0x241940 or f == idc.BADADDR:
        continue
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
out.close()
