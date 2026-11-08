import idc, idautils, ida_funcs, struct
out = open("D:/projects/project poland rewrite/research/gfx/s3_var.txt", "w", encoding="utf-8")
fn = ida_funcs.get_func(0xB826C)
prev = []
for ea in idautils.Heads(fn.start_ea, fn.end_ea):
    d = idc.generate_disasm_line(ea, 0)
    prev.append("   %X %s" % (ea, d))
    if "var_14], " in d or "[ebp-14h], " in d:
        out.write("\n".join(prev[-8:]) + "\n----\n")
for a in (0x685EA0, 0x6318D0, 0x62F658):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
out.close()
print("ok")
