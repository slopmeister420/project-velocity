# frontend agent: instructions with an immediate (fe_imm_args.txt: value start end ...) -> research/fe_imm.txt
import idc, idautils, ida_ua
base = r"D:\projects\project poland rewrite"
a = open(base + r"\tools\ida\fe_imm_args.txt").read().split()
val = int(a[0], 16)
with open(base + r"\research\fe_imm.txt", "w") as o:
    for k in range(1, len(a), 2):
        s, e = int(a[k], 16), int(a[k + 1], 16)
        for h in idautils.Heads(s, e):
            if not idc.is_code(idc.get_full_flags(h)):
                continue
            for n in range(3):
                if idc.get_operand_type(h, n) == idc.o_imm and (idc.get_operand_value(h, n) & 0xFFFFFFFF) == val:
                    o.write("%08X %s  | %s\n" % (h, idc.GetDisasm(h), idc.get_func_name(h)))
print("ok")
