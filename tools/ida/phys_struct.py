# physics agent: member offsets of the VehicleEntity type -> research/phys_ve_fields.txt
import ida_typeinf
ROOT = r"D:\projects\project poland rewrite\research"
tif = ida_typeinf.tinfo_t()
tif.get_named_type(None, "VehicleEntity")
udt = ida_typeinf.udt_type_data_t()
tif.get_udt_details(udt)
out = ["%s %d" % (m.name, m.offset // 8) for m in udt]
open(ROOT + r"\phys_ve_fields.txt", "w").write("\n".join(out) + "\n")
print(len(out))
