# list functions in the puddle range with their xrefs-from callers; dump to research/gfx/pud_funcs.txt
import idautils, idc, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/pud_funcs.txt"
with open(out, "w") as f:
    for ea in idautils.Functions(0x131600, 0x133C00):
        callers = sorted(set("%X" % ida_funcs.get_func(x.frm).start_ea for x in idautils.XrefsTo(ea) if ida_funcs.get_func(x.frm)))
        f.write("%08X %s <- %s\n" % (ea, idc.get_func_name(ea), " ".join(callers)))
    f.write("-- xrefs to dword_720840\n")
    for x in idautils.XrefsTo(0x720840):
        fn = ida_funcs.get_func(x.frm)
        f.write("%08X in %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
print("ok")
