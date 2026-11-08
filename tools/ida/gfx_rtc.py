import idc, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\rtc_desc.txt"
with open(OUT, "w", encoding="utf-8") as f:
    for name, a in [("162FF0", 0x1636E7), ("162D10", 0x162FB8), ("11F860", 0x1200CA), ("1205E0", 0x12082C)]:
        # descriptor may be unaligned; scan a few bytes forward for {count, ptr}
        for off in range(0, 12):
            d = a + off
            cnt = ida_bytes.get_dword(d)
            ptr = ida_bytes.get_dword(d + 4)
            if 0 < cnt < 16 and 0x11000 <= ptr < 0x7C0000:
                f.write("%s desc at %08X count %d vars %08X\n" % (name, d, cnt, ptr))
                for i in range(cnt):
                    v = ptr + 12 * i
                    off_ = ida_bytes.get_dword(v)
                    if off_ >= 0x80000000:
                        off_ -= 0x100000000
                    size = ida_bytes.get_dword(v + 4)
                    nm = ida_bytes.get_dword(v + 8)
                    s = idc.get_strlit_contents(nm, -1, 0) if nm else b""
                    f.write("   ebp%+d size %d %s\n" % (off_, size, s))
                break
print("ok")
