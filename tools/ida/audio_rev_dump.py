import idc, ida_funcs, ida_hexrays, idautils
# decompile the functions listed (hex, one per line, optional "a-b" range) in research/audio/review/req.txt
# to research/audio/review/dump.txt
D = r"D:\projects\project poland rewrite\research\audio\review"
eas = []
for line in open(D + r"\req.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    if "-" in line:
        a, b = [int(x, 16) for x in line.split("-")]
        eas += list(idautils.Functions(a, b))
    else:
        eas.append(int(line, 16))
with open(D + r"\dump.txt", "w", encoding="utf-8") as fo:
    for ea in eas:
        f = ida_funcs.get_func(ea)
        if not f:
            fo.write("// no func at %x\n" % ea)
            continue
        try:
            c = str(ida_hexrays.decompile(f.start_ea))
        except Exception as e:
            c = "// decompile failed: %s" % e
        fo.write("// ===== %x %s\n%s\n\n" % (f.start_ea, idc.get_func_name(f.start_ea), c))
print("dumped", len(eas))
