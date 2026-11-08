# vehicle database agent: misc lookups
import ida_name, idc
ea = ida_name.get_name_ea(idc.BADADDR, "VsClamp")
print(hex(ea))
f = open(r"D:\projects\project poland rewrite\research\vdb_lines_args.txt", "w"); f.write("%x %x\n" % (ea, idc.find_func_end(ea))); f.close()
