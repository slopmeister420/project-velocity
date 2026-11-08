# derived VsRender vtable: the ctor calling base ctor 0x57FD0 stores a vtable after it; decompile +28/+36
import ida_hexrays, idc, ida_bytes, idautils, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/s9_vt.c"
with open(out, "w", encoding="utf-8") as f:
    for x in idautils.XrefsTo(0x57FD0):
        fn = ida_funcs.get_func(x.frm)
        f.write("caller %08X %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
        h = x.frm
        for _ in range(20):
            h = idc.next_head(h)
            d = idc.GetDisasm(h)
            if "offset off_" in d or "offset ??_7" in d:
                f.write("   %08X %s\n" % (h, d))
                vt = idc.get_operand_value(h, 1)
                for off in (28, 36):
                    ea = ida_bytes.get_dword(vt + off)
                    f.write("//==== vt %08X +%d %08X %s\n" % (vt, off, ea, idc.get_func_name(ea)))
                    try:
                        f.write(str(ida_hexrays.decompile(ea)) + "\n")
                    except Exception as ex:
                        f.write("// fail %s\n" % ex)
                break
print("ok")
