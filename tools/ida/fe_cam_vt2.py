import idautils, idc
out = []
ea = 0x65AB38
while ea > 0x65A900:
    if list(idautils.DataRefsTo(ea)):
        out.append("vt start %X" % ea); break
    ea -= 4
base = ea
offs = {0x65AB30 - base: "add", 0x65AB34 - base: "get", 0x65AB38 - base: "setrefl"}
out.append(str(offs))
for f in idautils.Functions(0x200000, 0x2A0000):
    for h in idautils.FuncItems(f):
        if idc.print_insn_mnem(h) == "call":
            d = idc.generate_disasm_line(h, 0)
            for o, n in offs.items():
                if ("+%Xh]" % o) in d:
                    out.append("%X %s %s | %s" % (h, idc.get_func_name(f), n, d))
open(r"D:/projects/project poland rewrite/research/fe_cam_vt2.txt", "w").write("\n".join(out))
