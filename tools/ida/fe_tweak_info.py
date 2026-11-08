# frontend agent: callers of the frontend AddTweaks functions + initial values of the tweaked globals -> research/fe_tweak_info.txt
import idc, idautils, ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_tweak_info.txt", "w", encoding="utf-8")
for f in [0xF82F0, 0x1FDC20, 0x203450, 0x218390, 0x21CB70, 0x23B4A0, 0x247BE0, 0x258770, 0x26FC40, 0x202EA0, 0xBBEF0]:
    out.write("== %X %s\n" % (f, idc.get_func_name(f)))
    for x in idautils.XrefsTo(f):
        out.write("  from %X %s type %d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
glob = [0x6860EC, 0x6F719C, 0x6F71A0, 0x75CA88, 0x75CA58, 0x75CA68, 0x75CA78, 0x75CA38, 0x75CA3C, 0x75CA40, 0x75CA44,
        0x75CA48, 0x75CA4C, 0x6865E4, 0x75CA50, 0x75EDB4, 0x6F0D70, 0x75F44C, 0x75F44D, 0x75F44F, 0x7680B4, 0x7680B5,
        0x7680B6, 0x6C118C, 0x75C7A1, 0x736554, 0x6C1404, 0x6877F0, 0x76897C, 0x6C1510, 0x6C1514, 0x6C1518, 0x7691BC,
        0x7691BD, 0x7691BE, 0x7691BF, 0x6C1530, 0x7065A4, 0x6C1534, 0x769280, 0x769290, 0x7692A0, 0x7692B0, 0x7692C0]
for g in glob:
    seg = idc.get_segm_name(g)
    d = ida_bytes.get_bytes(g, 16) or b""
    i = struct.unpack("<4I", d) if len(d) == 16 else ()
    fl = struct.unpack("<4f", d) if len(d) == 16 else ()
    out.write("%X %s %s u32 %s f %s\n" % (g, seg, idc.get_name(g), " ".join("%08X" % v for v in i), " ".join("%g" % v for v in fl)))
    n = 0
    for x in idautils.XrefsTo(g):
        if n < 12:
            out.write("    xr %X %s t%d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
        n += 1
    out.write("    xrefs %d\n" % n)
out.close()
