# graphics queries: reads tools/ida/gfx_q_args.txt lines, writes research/gfx/q.out
#   strtab ADDR N   - N string pointers
#   u32 ADDR N / f32 ADDR N / str ADDR / dec ADDR (decompile one function) / dis ADDR N (instructions)
#   xref ADDR (code refs to an address, with containing function)
import idc, ida_bytes, ida_hexrays, ida_funcs, idautils, os, struct
base = os.path.dirname(os.path.abspath(__file__))
out = []
for line in open(os.path.join(base, "gfx_q_args.txt")):
    p = line.split()
    if not p:
        continue
    cmd, a = p[0], int(p[1], 16)
    n = int(p[2]) if len(p) > 2 else 1
    out.append("## " + line.strip())
    if cmd == "strtab":
        for i in range(n):
            q = ida_bytes.get_dword(a + 4 * i)
            s = idc.get_strlit_contents(q, -1, 0)
            out.append("%d %08X %s" % (i, q, s.decode("latin1") if s else None))
    elif cmd == "u32":
        out.append(" ".join("%08X" % ida_bytes.get_dword(a + 4 * i) for i in range(n)))
    elif cmd == "f32":
        out.append(" ".join(repr(struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(a + 4 * i)))[0]) for i in range(n)))
    elif cmd == "str":
        s = idc.get_strlit_contents(a, -1, 0)
        out.append(s.decode("latin1") if s else "None")
    elif cmd == "dec":
        try:
            out.append(str(ida_hexrays.decompile(a)))
        except Exception as e:
            out.append("fail %s" % e)
    elif cmd == "dis":
        ea = a
        for i in range(n):
            out.append("%08X %s" % (ea, idc.generate_disasm_line(ea, 0)))
            ea = idc.next_head(ea)
    elif cmd == "xref":
        for x in idautils.XrefsTo(a):
            f = ida_funcs.get_func(x.frm)
            out.append("%08X %s" % (x.frm, idc.get_func_name(f.start_ea) if f else "-"))
open(os.path.join(base, "..", "..", "research", "gfx", "q.out"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok")
# extra: "funcs START END" lists functions with their callers (appended mode, run via gfx_q2_args.txt)
p2 = os.path.join(base, "gfx_q2_args.txt")
if os.path.exists(p2):
    out2 = []
    for line in open(p2):
        q = line.split()
        if len(q) == 3 and q[0] == "funcs":
            for f in idautils.Functions(int(q[1], 16), int(q[2], 16)):
                cs = sorted({idc.get_func_name(x.frm) for x in idautils.XrefsTo(f) if ida_funcs.get_func(x.frm)})
                out2.append("%08X %s <- %s" % (f, idc.get_func_name(f), ", ".join(cs[:8])))
    open(os.path.join(base, "..", "..", "research", "gfx", "q2.out"), "w").write("\n".join(out2) + "\n")
