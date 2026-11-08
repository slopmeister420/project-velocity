# read f32 values at the hex addresses in research/ai_dump/_req.txt -> _xref.txt (ai agent)
import ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\ai_dump\_xref.txt", "w")
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_req.txt"):
    line = line.strip()
    if line:
        ea = int(line, 16)
        v = ida_bytes.get_dword(ea)
        out.write("%X %08X %r\n" % (ea, v, struct.unpack("<f", struct.pack("<I", v))[0]))
out.close()
