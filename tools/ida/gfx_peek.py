# print floats/dwords at addresses in tools/ida/gfx_peek_args.txt (first line = output path) and xrefs to struct offsets
import ida_bytes, idc, struct, os, idautils, ida_funcs, ida_ua
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_peek_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        if a.startswith("disp "):
            # disp <lo> <hi> <value>: instructions in [lo,hi) with displacement value
            _, lo, hi, v = a.split()
            lo, hi, v = int(lo, 16), int(hi, 16), int(v, 0)
            f.write("== disp %X in %X..%X\n" % (v, lo, hi))
            for h in idautils.Heads(lo, hi):
                insn = ida_ua.insn_t()
                if ida_ua.decode_insn(insn, h) == 0:
                    continue
                for op in insn.ops:
                    if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and op.addr == v:
                        fn = ida_funcs.get_func(h)
                        f.write("  %08X %s %s\n" % (h, idc.get_func_name(fn.start_ea) if fn else "?", idc.GetDisasm(h)))
            continue
        ea = int(a, 16)
        d = ida_bytes.get_dword(ea)
        f.write("%08X %s dword %08X float %r\n" % (ea, idc.get_name(ea), d, struct.unpack("<f", struct.pack("<I", d))[0]))
print("wrote", out)
