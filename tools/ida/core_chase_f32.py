import ida_bytes, struct
# floats of the follow camera globals -> research/core_chase_f32.txt
eas = list(range(0x686B58, 0x686B70, 4)) + [0x6851F0, 0x68522C, 0x7805E0, 0x7805E4, 0x7805E8]
out = []
for ea in eas:
    v = ida_bytes.get_dword(ea)
    out.append("%X %08X %r" % (ea, v, struct.unpack("<f", struct.pack("<I", v))[0]))
open(r"D:\projects\project poland rewrite\research\core_chase_f32.txt", "w").write("\n".join(out))
print("ok")
