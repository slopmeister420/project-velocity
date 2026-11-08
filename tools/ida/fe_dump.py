# frontend agent: decompile every function in the regions listed in fe_dump_args.txt
# args file lines: <start hex> <end hex> <outfile>
import os, idautils, idc, ida_hexrays, ida_funcs
here = os.path.dirname(os.path.abspath(__file__)) if "__file__" in dir() else r"D:\projects\project poland rewrite\tools\ida"
args = open(os.path.join(r"D:\projects\project poland rewrite\tools\ida", "fe_dump_args.txt")).read().split("\n")
for line in args:
    p = line.split(None, 2)
    if len(p) < 3:
        continue
    s, e, out = int(p[0], 16), int(p[1], 16), p[2].strip()
    with open(out, "w", encoding="utf-8") as fo:
        for f in idautils.Functions(s, e):
            fo.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
            try:
                fo.write(str(ida_hexrays.decompile(f)))
            except Exception as ex:
                fo.write("// decompile failed: %s\n" % ex)
            fo.write("\n\n")
    print("wrote", out)
