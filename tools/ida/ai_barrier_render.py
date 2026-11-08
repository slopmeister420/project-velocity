# barrier render: functions in its region, callers of 0x108030, 0x10B7F0/0x10B820 bodies (ai agent)
import idautils, ida_funcs, ida_name, ida_hexrays, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\ai_dump\_barrier_render.txt"
with open(OUT, "w") as o:
    for fea in idautils.Functions(0x106B30, 0x10CCB0):
        callers = sorted({hex(ida_funcs.get_func(x.frm).start_ea) for x in idautils.XrefsTo(fea) if ida_funcs.get_func(x.frm)})
        o.write("%08X %s size=%d callers=%s\n" % (fea, ida_name.get_name(fea), ida_funcs.get_func(fea).size(), ",".join(callers[:8])))
    o.write("dword_68656C = %r\n" % ida_bytes.get_dword(0x68656C))
    for ea in (0x10B7F0, 0x10B820):
        o.write(str(ida_hexrays.decompile(ea)) + "\n")
print("ok")
