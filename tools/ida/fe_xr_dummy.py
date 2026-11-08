# frontend agent: static init of dword_75EDA0[3] (MultiplayerScreen entries) -> research/fe_split_xr.txt
import idautils, idc
base = r"D:\projects\project poland rewrite"
out = open(base + r"\research\fe_split_xr.txt", "w", encoding="utf-8")
for a in range(0x75EDA0, 0x75EDAC, 4):
    for x in idautils.XrefsTo(a):
        if idc.print_insn_mnem(x.frm) != "mov" or not idc.print_operand(x.frm, 0).startswith("dword"):
            continue
        ea = x.frm
        for _ in range(8):
            ea = idc.prev_head(ea)
            v = idc.get_operand_value(ea, 0)
            s = idc.get_strlit_contents(v) if v > 0x600000 else None
            if s:
                out.write("%08X %08X %s\n" % (a, ea, s.decode("latin-1")))
                break
out.close()
