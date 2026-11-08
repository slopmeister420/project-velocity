# disassemble the functions listed in research/gfx/el_dis_in.txt -> el_dis_out.txt (float consts resolved)
import idautils, idc, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_dis_in.txt"):
    l = l.split("#")[0].strip()
    if not l:
        continue
    a = int(l, 16)
    out.append("==== %08X %s" % (a, idc.get_func_name(a)))
    for ea in idautils.FuncItems(a):
        s = idc.GetDisasm(ea)
        extra = ""
        for i in range(2):
            if idc.get_operand_type(ea, i) == idc.o_mem:
                v = idc.get_operand_value(ea, i)
                f = struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(v)))[0]
                extra += "  ; [%X]=%r/0x%08X" % (v, f, ida_bytes.get_dword(v))
        out.append("%06X %s%s" % (ea, s, extra))
open(B + r"\el_dis_out.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
