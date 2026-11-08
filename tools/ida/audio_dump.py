import os, idc, ida_funcs, ida_hexrays
# decompile the functions listed (hex, one per line, optional "a-b" range) in audio_req.txt to audio_dump.txt
SP = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "research", "audio")
eas = []
for line in open(SP + r"\audio_req.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    if "-" in line:
        a, b = [int(x, 16) for x in line.split("-")]
        import idautils
        eas += list(idautils.Functions(a, b))
    else:
        eas.append(int(line, 16))
with open(SP + r"\audio_dump.txt", "w", encoding="utf-8") as fo:
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
