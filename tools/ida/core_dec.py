import idc, ida_hexrays, ida_name
# decompile the functions listed in research/core_dec_in.txt (hex or names, one per line) to research/core_dec.txt
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_dec.txt", "w")
for line in open(base + r"\core_dec_in.txt"):
    s = line.strip()
    if not s:
        continue
    try:
        ea = int(s, 16)
    except ValueError:
        ea = ida_name.get_name_ea(idc.BADADDR, s)
    f = idc.get_func_attr(ea, idc.FUNCATTR_START)
    try:
        out.write("\n==== %x %s\n%s\n" % (f, idc.get_func_name(f), str(ida_hexrays.decompile(f))))
    except Exception as e:
        out.write("\n==== %s fail %s\n" % (s, e))
out.close()
