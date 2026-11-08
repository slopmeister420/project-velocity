# find the char element vtable (data refs to its Init 0x284A30 / update 0x285350), decompile slot +96 and its callees
import idautils, idc, ida_hexrays, ida_bytes
out = r"D:/projects/project poland rewrite/research/fe_spec764d.c"
with open(out, "w", encoding="utf-8") as f:
    for tgt in (0x284A30, 0x285350, 0x284C20):
        for x in idautils.XrefsTo(tgt):
            f.write("// ref to %08X from %08X seg %s name %s\n" % (tgt, x.frm, idc.get_segm_name(x.frm), idc.get_name(x.frm)))
    # vtable guess: data ref to 0x284A30
    vt = None
    for x in idautils.XrefsTo(0x284A30):
        if not idc.get_func_name(x.frm):
            vt = x.frm
    f.write("// slot of 284A30 at %s\n" % (("%08X" % vt) if vt else "none"))
    if vt:
        # walk back to start
        s = vt
        while ida_bytes.get_dword(s - 4) and 0x11000 <= ida_bytes.get_dword(s - 4) < 0x3DDD20 and not idc.get_name(s):
            s -= 4
        f.write("// vtable start %08X %s\n" % (s, idc.get_name(s)))
        for k in range(40):
            f.write("//  +%d %08X %s\n" % (4 * k, ida_bytes.get_dword(s + 4 * k), idc.get_func_name(ida_bytes.get_dword(s + 4 * k))))
        slot = ida_bytes.get_dword(s + 96)
        for ea in (slot,):
            f.write("// ===== slot96 %08X\n" % ea)
            try:
                f.write(str(ida_hexrays.decompile(ea)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)
print("wrote", out)
