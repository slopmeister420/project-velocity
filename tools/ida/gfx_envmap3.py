# dynamic envmap stage 3: draw/update callers, helpers, constants -> research/gfx/envmap3_*.c/txt
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
main = [0x159D00, 0x15A090, 0x1598A0, 0x10CE90, 0x1569F5, 0x157219, 0x113BD2]
helpers = [0x11BA10, 0x11BA30, 0x2AC070, 0x2AC430, 0x2AC450, 0x42CD0, 0x7A840, 0x5ABB0, 0x11DFF0, 0x57F90,
           0x613E0, 0x5B0F0, 0x5ACD0, 0x2EF030, 0x29F7F0, 0x2EEA10, 0x761E0, 0x2A05D0]
out = []
seen = set()
def dec(ea, tag):
    f = ida_funcs.get_func(ea)
    if f is None or f.start_ea in seen:
        return
    seen.add(f.start_ea)
    callers = ["%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(f.start_ea)]
    s = "// ===== %s %X..%X %s callers: %s\n" % (tag, f.start_ea, f.end_ea, idc.get_func_name(f.start_ea), " ".join(callers[:10]))
    try:
        s += str(ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        s += "// fail %s" % ex
    out.append(s)
for ea in main:
    dec(ea, "main")
# callers of the main draw functions (one level up)
for ea in [0x159D00, 0x15A090, 0x1598A0]:
    for x in idautils.XrefsTo(ea):
        dec(x.frm, "caller")
open(B + r"\envmap3_main.c", "w", encoding="utf-8").write("\n".join(out))
out = []
for ea in helpers:
    dec(ea, "helper")
open(B + r"\envmap3_help.c", "w", encoding="utf-8").write("\n".join(out))
# constants 0x686920..0x6869A4
c = []
for a in range(0x686920, 0x6869A4, 4):
    v = ida_bytes.get_dword(a)
    c.append("%X %08X %s %s" % (a, v, repr(struct.unpack("<f", struct.pack("<I", v))[0]), idc.get_name(a)))
for a in [0x6F4AC8, 0x624E14, 0x718E78, 0x7190F0, 0x68697C]:
    c.append("%X %08X" % (a, ida_bytes.get_dword(a)))
open(B + r"\envmap3_consts.txt", "w", encoding="utf-8").write("\n".join(c))
print("ok")
