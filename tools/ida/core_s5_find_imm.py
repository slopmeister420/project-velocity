import idautils, idc, ida_bytes
# find any 4-byte occurrence of 0x000BDD20 in the database -> research/core_s5_imm.txt
out = open(r"D:\projects\project poland rewrite\research\core_s5_imm.txt", "w")
ea = 0
pat = bytes([0x20, 0xDD, 0x0B, 0x00])
for seg in idautils.Segments():
    s, e = idc.get_segm_start(seg), idc.get_segm_end(seg)
    b = ida_bytes.get_bytes(s, e - s) or b""
    i = b.find(pat)
    while i >= 0:
        out.write("%X %s\n" % (s + i, idc.get_func_name(s + i)))
        i = b.find(pat, i + 1)
out.close()
