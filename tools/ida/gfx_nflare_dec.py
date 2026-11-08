# near flares + road reflections -> research/gfx/nflare_dec.c (helpers) + nflare_vals.txt (constants and refs)
import ida_hexrays, idc, idautils, ida_bytes, ida_name, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
fs = [0x11DF60, 0x11DFF0, 0x5ACD0, 0x5AC40, 0x5ABB0, 0x11BA30, 0x611B0, 0x2ABD60, 0x2ABF70, 0x452C0,
      0x2B11C0, 0x43FB0, 0x149020, 0x149480, 0x14A010, 0x148EE0, 0x148F90]
for a in fs:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
    out.append("// callers: " + " ".join("%X(%s)" % (x, idc.get_func_name(x)) for x in list(idautils.CodeRefsTo(a, 0))[:12]))
open(B + r"\nflare_dec.c", "w", encoding="utf-8").write("\n".join(out))
v = []
for lo, hi in [(0x6872A0, 0x687340)]:
    for a in range(lo, hi, 4):
        d = ida_bytes.get_dword(a)
        v.append("%X = %08X %r %s" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0], ida_name.get_name(a)))
        for k in range(4):
            for x in idautils.DataRefsTo(a + k):
                v.append("   <- +%d %X %s | %s" % (k, x, idc.get_func_name(x), idc.GetDisasm(x)))
open(B + r"\nflare_vals.txt", "w", encoding="utf-8").write("\n".join(v))
print("ok")
