# generic query driven by tools/ida/gfx2_q.txt; lines:
#   dc <out> <addr> [addr...]          decompile functions (containing addr) into research/<out>
#   xr <out> <addr> [addr...]          xrefs to addr (with function names) appended to research/<out>
#   rng <out> <start> <end>            decompile every function in range
#   str <out> <regex>                  strings matching regex with their xref functions
import idautils, ida_hexrays, ida_funcs, idc, os, re
root = r"D:\projects\project poland rewrite"
base = os.path.dirname(os.path.abspath(__file__))
def dc(a):
    f = ida_funcs.get_func(a)
    if not f:
        return "// no func %X\n" % a
    try:
        return "//==== %08X %s\n%s\n" % (f.start_ea, idc.get_func_name(f.start_ea), ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        return "//==== %08X fail %s\n" % (f.start_ea, ex)
strs = None
for line in open(os.path.join(base, "gfx2_q.txt")):
    p = line.split()
    if not p or p[0].startswith("#"):
        continue
    out = os.path.join(root, "research", p[1])
    with open(out, "w", encoding="utf-8") as o:
        if p[0] == "dc":
            for a in p[2:]:
                o.write(dc(int(a, 16)))
        elif p[0] == "rng":
            for fa in idautils.Functions(int(p[2], 16), int(p[3], 16)):
                o.write(dc(fa))
        elif p[0] == "xr":
            for a in p[2:]:
                for x in idautils.XrefsTo(int(a, 16)):
                    f = ida_funcs.get_func(x.frm)
                    o.write("%s %08X %s\n" % (a, x.frm, f and idc.get_func_name(f.start_ea)))
        elif p[0] == "str":
            rx = re.compile(" ".join(p[2:]), re.I)
            if strs is None:
                strs = list(idautils.Strings())
            for s in strs:
                t = str(s)
                if rx.search(t):
                    fs = set()
                    for x in idautils.XrefsTo(s.ea):
                        f = ida_funcs.get_func(x.frm)
                        fs.add("%08X:%s" % (x.frm, f and idc.get_func_name(f.start_ea)))
                    o.write("%08X %r %s\n" % (s.ea, t, " ".join(sorted(fs))))
    print("wrote", out)
