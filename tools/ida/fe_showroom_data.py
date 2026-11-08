# frontend agent: showroom data words + writers -> research/fe_showroom_data.txt
import idc, idautils, ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_showroom_data.txt", "w")
for a in [0x75F6D0, 0x75F6D4, 0x75F6D8, 0x77FD50, 0x77FD9C, 0x6C151C, 0x6C1520, 0x6877F0, 0x75F44C]:
    x = ida_bytes.get_dword(a)
    out.write("%08X = %08X (%g)\n" % (a, x, struct.unpack("<f", struct.pack("<I", x))[0]))
    for r in idautils.XrefsTo(a):
        out.write("   xref %08X %s %s\n" % (r.frm, idc.get_func_name(r.frm), idc.GetDisasm(r.frm)))
# the static init writing 75F6D0..: dump a few lines around each write
for a in [0x75F6D0, 0x75F6D4, 0x75F6D8, 0x77FD50, 0x77FD9C]:
    for r in idautils.XrefsTo(a):
        ea = r.frm
        p = ea
        for _ in range(8):
            p = idc.prev_head(p)
        while p <= ea:
            out.write("      %08X %s\n" % (p, idc.GetDisasm(p)))
            p = idc.next_head(p)
        break
out.close()
print("ok")
