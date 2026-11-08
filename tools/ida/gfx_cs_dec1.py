# fixed-function car sets: helpers -> research/gfx/cs_dec1.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
names = ["CollisionMeshRender__11F260", "ShaderManagerBase__2A05D0", "ShaderManagerBase__2A1AC0",
         "ShaderManagerBase__2A2100", "ShaderManagerBase__2A1DC0", "ShaderManagerBase__2A1990",
         "ShaderManagerBase__2A1A00", "ShaderManagerBase__2A1240", "A1::ShaderManagerBase",
         "CarShaderManager_PC_XBOX__158AF0", "CarShaderManager_PC_XBOX__158BB0",
         "CarShaderManager_PC_XBOX__158C10", "CarShaderManager_PC_XBOX__158B90",
         "CShaderManagerBase::SetStageFlags", "CTextureManager::Register", "sub_4B023B", "sub_445660",
         "Armco__13F50", "CShaderManagerBase::SetColourOp", "CShaderManagerBase::SetAlphaOp",
         "ShaderManagerBase__2A1BE0", "CShaderManagerBase::SetTexture"]
for n in names:
    ea = idc.get_name_ea_simple(n)
    if ea == idc.BADADDR:
        out.append("// missing %s" % n)
        continue
    out.append("//==== %X %s" % (ea, n))
    try:
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
# vtables of the set objects: dump the ctor-assigned vtable entries
for ctor in (0x158BB0, 0x158C10):
    f = ida_funcs.get_func(ctor)
    ea = f.start_ea
    while ea < f.end_ea:
        if idc.print_insn_mnem(ea) == "mov" and idc.get_operand_type(ea, 1) == idc.o_imm:
            v = idc.get_operand_value(ea, 1)
            if 0x600000 < v < 0x700000:
                ents = []
                for k in range(4):
                    t = idc.get_wide_dword(v + 4 * k)
                    ents.append("%X %s" % (t, idc.get_func_name(t)))
                out.append("// ctor %X vtable %X: %s" % (ctor, v, " | ".join(ents)))
        ea = idc.next_head(ea)
# who reads palette +0 (CarColour handle): xrefs of 1561C0 and 157F80
for t in (0x1561C0, 0x157F80, 0x157BA0, 0x157D40, 0x156150):
    for x in idautils.XrefsTo(t):
        out.append("// %X caller %X in %s" % (t, x.frm, idc.get_func_name(x.frm)))
open(B + r"\cs_dec1.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
