# neon underglow 0x111B20 helpers -> research/gfx/neon_dec.c, grep of car field users -> neon_grep.txt
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes, re
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x115920, 0x11B790, 0x1C68F0, 0x364BB0, 0x57F90, 0x2AB250, 0x19D350, 0x5C820]:
    f = ida_funcs.get_func(ea)
    if not f:
        out.append("// nofunc %X" % ea); continue
    out.append("//==== %08X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\neon_dec.c", "w", encoding="utf-8").write("\n".join(out))
# data values
vals = []
for name, ea, n in [("686780", 0x686780, 4), ("68677C", 0x68677C, 1), ("686778", 0x686778, 1), ("7184B0", 0x7184B0, 1),
                    ("6865E8", 0x6865E8, 1), ("6865F4", 0x6865F4, 1), ("6865F8", 0x6865F8, 1), ("7065A4", 0x7065A4, 1)]:
    for i in range(n):
        a = ea + 4 * i
        v = ida_bytes.get_dword(a)
        import struct
        vals.append("%X %08X int %d float %r" % (a, v, v if v < 0x80000000 else v - 0x100000000, struct.unpack("<f", struct.pack("<I", v))[0]))
for ea in [0x709ECC, 0x709ED8, 0x7065A4, 0x7184B0, 0x686778, 0x68677C, 0x6865E8]:
    for x in idautils.XrefsTo(ea):
        vals.append("xref %X <- %X %s : %s" % (ea, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
open(B + r"\neon_vals.txt", "w", encoding="utf-8").write("\n".join(vals))
# grep car field offsets in CarRender + VehicleEntity + GameDataLoading range decompilations
pat = re.compile(r"\+ (1808|1812|1816|2232|2328|1820|932|1804)\)")
g = []
for (s, e) in [(0x10CCB0, 0x11CDB0), (0x155BF0, 0x1581F0)]:
    for fea in idautils.Functions(s, e):
        try:
            txt = str(ida_hexrays.decompile(fea))
        except Exception:
            continue
        for ln in txt.split("\n"):
            if pat.search(ln):
                g.append("%X %s: %s" % (fea, idc.get_func_name(fea), ln.strip()))
open(B + r"\neon_grep.txt", "w", encoding="utf-8").write("\n".join(g))
print("ok", len(g))
