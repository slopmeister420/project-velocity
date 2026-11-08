# frontend agent: dump element vtables
import ida_bytes, idc
out = open(r"D:\projects\project poland rewrite\research\fe_vt2.txt", "w")
import sys
for vt in [int(x,16) for x in open(r"D:\projects\project poland rewrite\research\fe_vt2_args.txt").read().split()]:
    out.write("== %08X\n" % vt)
    for i in range(20):
        f = ida_bytes.get_dword(vt + 4*i)
        out.write("  %d %08X %s\n" % (i, f, idc.get_func_name(f)))
out.close()
