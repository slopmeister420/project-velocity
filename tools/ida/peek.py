import idautils, idc, ida_hexrays, ida_funcs
want="VehicleEntity.cpp"
ea=[s.ea for s in idautils.Strings() if str(s).endswith("\Physics\Vehicle\VehicleEntity.cpp")][0]
fs=set()
for x in idautils.XrefsTo(ea):
    f=ida_funcs.get_func(x.frm)
    if f: fs.add(f.start_ea)
print("str", hex(ea), "xref funcs", len(fs))
f=sorted(fs)[3]
print(str(ida_hexrays.decompile(f))[:3500])
