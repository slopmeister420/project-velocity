# physics agent stage 4b: instructions in the object-manager range touching +0xC90 (m_CurrentList) / +0xA84
import idc, idautils
out = []
ea = 0x1DE730
while ea < 0x1E2670:
    d = idc.generate_disasm_line(ea, 0)
    if "0C90h]" in d or "0A84h]" in d:
        out.append("%X %s | %s" % (ea, idc.get_func_name(ea), d))
    ea = idc.next_head(ea, 0x1E2670)
open(r"D:\projects\project poland rewrite\research\phys_s4b_find.txt", "w").write("\n".join(out))
