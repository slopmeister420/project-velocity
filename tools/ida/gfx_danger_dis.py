import idc, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\danger_dis.txt"

def dis(f, s, e):
    a = s
    while a < e:
        f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
        a = idc.next_head(a, e + 16)

with open(OUT, "w", encoding="utf-8") as f:
    for a in [0x685E8C, 0x625180, 0x625678, 0x6249C0, 0x624968]:
        f.write("data %08X = %08X byte %d loaded %d\n" % (a, ida_bytes.get_dword(a), ida_bytes.get_byte(a), ida_bytes.is_loaded(a)))
    for a in range(0x718AC0 + 72, 0x718AC0 + 232, 4):
        pass
    f.write("// 11F860 inner loop\n")
    dis(f, 0x11FDB0, 0x11FE80)
    f.write("// 1205E0 lerp part\n")
    dis(f, 0x120640, 0x120720)
    f.write("// 120100 tail\n")
    dis(f, 0x120280, 0x1202F0)
print("ok")
