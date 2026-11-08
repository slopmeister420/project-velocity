import idc, idautils, ida_funcs
out = []
for f in idautils.Functions(0x299000, 0x29A700):
    for h in idautils.FuncItems(f):
        d = idc.generate_disasm_line(h, 0)
        if ("+124h]" in d or "+104h]" in d or "+70h]" in d) and "ebp" not in d:
            out.append("%X %s | %s" % (h, idc.get_func_name(f), d))
open(r"D:/projects/project poland rewrite/research/fe_cam292.txt", "w").write("\n".join(out))
