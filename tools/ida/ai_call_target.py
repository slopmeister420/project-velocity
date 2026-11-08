# decompile the call target at the address in research/ai_dump/_req.txt (ai agent)
import idc, ida_hexrays
ea = int(open(r"D:\projects\project poland rewrite\research\ai_dump\_req.txt").read().split()[0], 16)
t = idc.get_operand_value(ea, 0)
out = open(r"D:\projects\project poland rewrite\research\ai_dump\_xref.txt", "w")
out.write("// %X\n" % t)
try:
    out.write(str(ida_hexrays.decompile(t, flags=ida_hexrays.DECOMP_NO_CACHE)))
    c = ida_hexrays.decompile(t, flags=ida_hexrays.DECOMP_NO_CACHE)
except Exception as e:
    out.write(str(e))
out.close()
