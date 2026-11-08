import idc, ida_funcs, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\ls_stencil4.txt"
with open(OUT, "w", encoding="utf-8") as f:
    fn = ida_funcs.get_func(0x12E5C0)
    a = fn.start_ea
    while a < fn.end_ea:
        l = idc.generate_disasm_line(a, 0)
        if "call" in l or "push" in l:
            f.write("%08X  %s\n" % (a, l))
        a = idc.next_head(a, fn.end_ea)
    f.write("71FF10 seg %s\n" % idc.get_segm_name(0x71FF10))
    f.write("71FF10 init %08X loaded %d\n" % (ida_bytes.get_dword(0x71FF10), ida_bytes.is_loaded(0x71FF10)))
print("ok")
