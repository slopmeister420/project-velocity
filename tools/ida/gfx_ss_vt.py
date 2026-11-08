# texture manager (dword_77FD30): find the vtable via the stores to it, decompile slots +20/+24
import ida_hexrays, idc, idautils, ida_bytes, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/wh_tm.c"
with open(out, "w", encoding="utf-8") as f:
    seen = set()
    for x in idautils.XrefsTo(0x77FD30):
        d = idc.GetDisasm(x.frm)
        f.write("// %08X %s\n" % (x.frm, d))
        if not d.startswith("mov     dword_77FD30"):
            continue
        fn = ida_funcs.get_func(x.frm)
        for h in idautils.Heads(fn.start_ea, fn.end_ea):
            dd = idc.GetDisasm(h)
            if "offset off_" in dd and "mov" in dd:
                v = idc.get_operand_value(h, 1)
                if v in seen:
                    continue
                seen.add(v)
                f.write("// vt %08X in %s\n" % (v, idc.get_func_name(fn.start_ea)))
                for off in (20, 24):
                    ea = ida_bytes.get_dword(v + off)
                    f.write("//==== +%d %08X %s\n" % (off, ea, idc.get_func_name(ea)))
                    try:
                        f.write(str(ida_hexrays.decompile(ea)) + "\n")
                    except Exception as ex:
                        f.write("// fail %s\n" % ex)
print("wrote", out)
