import idc, idautils, ida_bytes
# xrefs to the bink runtime tables, plus the BINKDATA bytes
base = r"D:\projects\project poland rewrite\research"
out = []
for a in (0x7D2920, 0x7D2EF8, 0x7D2EFC, 0x7D2F10, 0x7D2F50, 0x7D2F90, 0x7D2FA0, 0x7D3247, 0x7D3100, 0x7D4120, 0x7D5188,
          0x7D2700, 0x7D2880, 0x7D2698, 0x7D2960, 0x7D29A0, 0x7D2A0C):
    refs = ["%X(%s)" % (r, idc.get_func_name(r)) for r in idautils.DataRefsTo(a)]
    out.append("%X: %s" % (a, " ".join(refs)))
open(base + r"\fe_bink_tabs.txt", "w").write("\n".join(out))
data = ida_bytes.get_bytes(0x7CEC20, 0x7D2A78 - 0x7CEC20)
open(base + r"\fe_bink_data.bin", "wb").write(data)
data = ida_bytes.get_bytes(0x543585, 0x5443E0 - 0x543585)
open(base + r"\fe_bink_gap.bin", "wb").write(data)
