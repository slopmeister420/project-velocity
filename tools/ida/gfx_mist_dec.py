# puddle mist -> research/gfx/mist_dec.c (helpers, per-frame funcs) + mist_vals.txt (constants and refs)
import ida_hexrays, idc, idautils, ida_bytes, ida_name, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
cf = ida_name.get_name_ea(idc.BADADDR, "CF::elParticleSystem")
fs = [0x1339B0, 0x5C820, 0x79030, 0x79640, 0x79470, 0x1341A0, 0x135690, 0x133910]
if cf != idc.BADADDR:
    fs.append(cf)
for a in fs:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
    out.append("// callers: " + " ".join("%X(%s)" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(a, 0)))
open(B + r"\mist_dec.c", "w", encoding="utf-8").write("\n".join(out))
v = []
rngs = [(0x686EA8, 0x686F20), (0x7207C0, 0x720840), (0x721B10, 0x721B60)]
for lo, hi in rngs:
    for a in range(lo, hi, 4):
        d = ida_bytes.get_dword(a)
        v.append("%X = %08X %r %s" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0], ida_name.get_name(a)))
        for x in idautils.DataRefsTo(a):
            v.append("   <- %X %s | %s" % (x, idc.get_func_name(x), idc.GetDisasm(x)))
open(B + r"\mist_vals.txt", "w", encoding="utf-8").write("\n".join(v))
print("ok", hex(cf))
