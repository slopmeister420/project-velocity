# frontend agent: decompile every function in ranges / single addresses from fe_dcr_args.txt
# lines: "<out file> <start>[-<end>] ..." -> research/<out file>
import idc, idautils, ida_hexrays, ida_name
base = r"D:\projects\project poland rewrite"
for line in open(base + r"\tools\ida\fe_dcr_args.txt").read().splitlines():
    p = line.split()
    if not p:
        continue
    out = open(base + "\\research\\" + p[0], "w", encoding="utf-8")
    eas = []
    for t in p[1:]:
        if "-" in t:
            s, e = [int(x, 16) for x in t.split("-")]
            eas += [f for f in idautils.Functions(s, e)]
        else:
            try:
                ea = int(t, 16)
            except ValueError:
                ea = ida_name.get_name_ea(idc.BADADDR, t)
            eas.append(idc.get_func_attr(ea, idc.FUNCATTR_START))
    for f in eas:
        out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
        try:
            out.write(str(ida_hexrays.decompile(f)) + "\n\n")
        except Exception as ex:
            out.write("// failed %s\n" % ex)
    out.close()
print("ok")
