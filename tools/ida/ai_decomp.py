import ida_hexrays, idc
# decompile the functions listed in research/ai_decomp_in.txt (hex addrs) into research/ai_decomp_out.c
base = r"D:\projects\project poland rewrite\research"
addrs = [int(l.strip(), 16) for l in open(base + r"\ai_decomp_in.txt") if l.strip()]
out = []
for a in addrs:
    try:
        out.append("// ---- %X %s\n%s" % (a, idc.get_func_name(a), str(ida_hexrays.decompile(a))))
    except Exception as e:
        out.append("// ---- %X failed %s" % (a, e))
open(base + r"\ai_decomp_out.c", "w").write("\n".join(out))
