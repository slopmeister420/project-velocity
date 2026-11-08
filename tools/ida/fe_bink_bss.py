import ida_bytes, idc, ida_segment
# dump BINKBSS bytes and check whether they are loaded (initialised) in the idb
base = r"D:\projects\project poland rewrite\research"
s = ida_segment.getseg(0x7D2A80)
n = 0x7D5588 - 0x7D2A80
data = ida_bytes.get_bytes(0x7D2A80, n)
loaded = sum(1 for i in range(0, n, 4) if ida_bytes.is_loaded(0x7D2A80 + i))
open(base + r"\fe_bink_bss.bin", "wb").write(data if data else b"")
open(base + r"\fe_bink_bss.txt", "w").write("len=%d loaded_dwords=%d nonzero=%d" % (n, loaded, sum(1 for b in (data or b"") if b)))
