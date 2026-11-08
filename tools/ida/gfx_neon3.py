# users of the neon / shadow shader strings + 63B730 bytes -> research/gfx/neon_dec3.c
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
out.append("// 63B730 bytes %s" % ida_bytes.get_bytes(0x63B730, 16).hex())
out.append("// 63B730 str %r" % idc.get_strlit_contents(0x63B730))
funcs = set()
for s in [0x63AF54, 0x63AF60, 0x63AF90, 0x63AFA0]:
    for x in idautils.XrefsTo(s):
        out.append("// str %X used at %X in %s" % (s, x.frm, idc.get_func_name(x.frm)))
        f = ida_funcs.get_func(x.frm)
        if f:
            funcs.add(f.start_ea)
for ea in sorted(funcs):
    out.append("//==== %08X %s" % (ea, idc.get_func_name(ea)))
    try:
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\neon_dec3.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
