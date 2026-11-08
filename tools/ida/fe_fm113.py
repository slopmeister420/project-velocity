# frontend agent: writes of "byte [reg+71h]" (CFrontendManager +113) in the FE range -> research/fe_fm113.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_fm113.txt", "w")
for fea in idautils.Functions(0x1FA000, 0x2B0000):
    for h in idautils.FuncItems(fea):
        d = idc.GetDisasm(h)
        if d.startswith("mov") and "+71h]" in d and "byte" in d and idc.get_operand_type(h, 0) in (3, 4):
            out.write("%08X %s  %s\n" % (h, idc.get_func_name(h), d))
out.close()
