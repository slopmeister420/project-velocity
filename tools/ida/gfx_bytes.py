# find raw dword occurrences of addresses (catches undefined code): args in tools/ida/gfx_bytes_args.txt
import ida_bytes, idc, ida_funcs, ida_segment, os, struct
base = os.path.dirname(os.path.abspath(__file__))
out = []
for line in open(os.path.join(base, "gfx_bytes_args.txt")):
    if not line.strip():
        continue
    v = int(line.split()[0], 16)
    pat = struct.pack("<I", v)
    out.append("## %08X" % v)
    s = ida_segment.get_first_seg()
    while s:
        data = ida_bytes.get_bytes(s.start_ea, s.end_ea - s.start_ea) or b""
        i = data.find(pat)
        while i >= 0:
            ea = s.start_ea + i
            f = ida_funcs.get_func(ea)
            out.append("%08X %s | %s" % (ea, idc.get_func_name(f.start_ea) if f else "-", idc.generate_disasm_line(idc.get_item_head(ea), 0)))
            i = data.find(pat, i + 1)
        s = ida_segment.get_next_seg(s.start_ea)
open(os.path.join(base, "..", "..", "research", "gfx", "bytes.out"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok")
