# uv anim object used by zonetrailone/two -> research/gfx/tb_uv.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
vt = 0x66B34C
fns = [0x3011A0, 0x301210, 0x302380, 0x302500]
for i in range(8):
    f = idc.get_wide_dword(vt + 4 * i)
    out.append("vt[%d] = %X %s" % (i, f, idc.get_func_name(f)))
    if ida_funcs.get_func(f):
        fns.append(f)
# who reads the stage +120 (85*4*stage + 30*4) uv anim pointer: callers of the vtable slots
for a in fns:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\tb_uv.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
