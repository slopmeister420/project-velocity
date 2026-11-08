# xrefs to addresses in gfx_s3_xref.txt ("addr" per line); decompile each referencing function once
import idautils, ida_hexrays, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
out = "D:/projects/project poland rewrite/research/gfx/s3_xref.c"
seen = set()
with open(out, "w", encoding="utf-8") as f:
    for line in open(os.path.join(base, "gfx_s3_xref.txt")):
        p = line.split()
        if not p:
            continue
        a = int(p[0], 16)
        f.write("//#### xrefs to %08X\n" % a)
        for x in idautils.XrefsTo(a):
            fn = ida_funcs.get_func(x.frm)
            fs = fn.start_ea if fn else 0
            f.write("// from %08X in %08X %s\n" % (x.frm, fs, idc.get_func_name(fs) if fn else "?"))
            if fn and fs not in seen and len(p) < 2:
                seen.add(fs)
                try:
                    f.write(str(ida_hexrays.decompile(fs)) + "\n")
                except Exception as ex:
                    f.write("// fail %s\n" % ex)
print("wrote", out, len(seen))
