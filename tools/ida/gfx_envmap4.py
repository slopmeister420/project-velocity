# dynamic envmap stage 4: render target / fse helpers, car shader manager getters, data -> research/gfx/envmap4_*
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
eas = [0x11B1C0, 0x159C10, 0x2EF0B0, 0x2EED60, 0x2EF160, 0x2EEFE0, 0x2EEE20, 0x15C340, 0x23DD0, 0x5AC40,
       0x7DD20, 0x12D2C0, 0x2AC0A0, 0x48A0F9, 0x158A40, 0x158AF0, 0x158B10, 0x2A1E80, 0x2A3230, 0x2A15F0,
       0x2A0E40, 0x5FFB0, 0x15E7B0]
out = []
for ea in eas:
    f = ida_funcs.get_func(ea)
    if f is None:
        out.append("// no func %X" % ea)
        continue
    callers = ["%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(f.start_ea)]
    s = "// ===== %X..%X %s callers(%d): %s\n" % (f.start_ea, f.end_ea, idc.get_func_name(f.start_ea), len(callers), " ".join(callers[:6]))
    try:
        s += str(ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        s += "// fail %s" % ex
    out.append(s)
open(B + r"\envmap4_help.c", "w", encoding="utf-8").write("\n".join(out))
c = []
for a in list(range(0x687540, 0x687584, 4)) + [0x6865EC, 0x7A5780]:
    v = ida_bytes.get_dword(a)
    c.append("%X %08X %s %s" % (a, v, repr(struct.unpack("<f", struct.pack("<I", v))[0]), idc.get_name(a)))
# who writes byte_687570 / 687558 / 686948
for a in [0x687570, 0x687558, 0x686948, 0x718E78, 0x7190F0, 0x687568, 0x68756C]:
    for x in idautils.XrefsTo(a):
        c.append("xref %X <- %X %s : %s" % (a, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
open(B + r"\envmap4_consts.txt", "w", encoding="utf-8").write("\n".join(c))
print("ok")
