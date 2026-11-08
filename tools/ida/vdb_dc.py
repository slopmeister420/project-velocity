# vehicle database agent: decompile addresses listed in research/vdb_dc_args.txt to research/vdb_dc_out.c
import ida_hexrays, ida_name, ida_funcs
ROOT = r"D:\projects\project poland rewrite\research"
addrs = [int(x, 16) for x in open(ROOT + r"\vdb_dc_args.txt").read().split()]
with open(ROOT + r"\vdb_dc_out.c", "w", encoding="utf-8") as o:
    for a in addrs:
        f = ida_funcs.get_func(a)
        s = f.start_ea if f else a
        o.write("\n// ==== %08X %s\n" % (s, ida_name.get_name(s)))
        try:
            o.write(str(ida_hexrays.decompile(s)))
        except Exception as ex:
            o.write("// failed %s\n" % ex)
print("ok", len(addrs))
