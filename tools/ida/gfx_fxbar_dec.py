import ida_hexrays, idc, idautils, ida_funcs
R = r"D:\projects\project poland rewrite\research\gfxdmg"

def dec_list(out, fns):
    with open(out, "w", encoding="utf-8") as f:
        for fa in fns:
            f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
            try:
                f.write(str(ida_hexrays.decompile(fa)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)

def funcs_in(a, b):
    return [x for x in idautils.Functions(a, b)]

# glass / damage fx (CarDamage.cpp tail + CarDamageFX.cpp)
glass = [0x781C0, 0x78270] + funcs_in(0x78320, 0x78880)
dec_list(R + r"\damagefx_glass.c", glass)
helpers = [0x2D4A70, 0x2D4BD0, 0x2D5A60, 0x2D4840, 0x2D59B0, 0x2D5800, 0x2D5960, 0x2D77C0, 0x2D78D0,
           0x2AB330, 0x2ABDA0, 0x2ABDE0, 0x2AB560, 0x2AC070, 0x14000, 0x13FA0, 0x2B11C0, 0x761B0, 0x79730]
dec_list(R + r"\damagefx_helpers.c", helpers)
# callers of the fx entry points and the cooldown
with open(R + r"\damagefx_xrefs.txt", "w") as f:
    for a in [0x78880, 0x781C0, 0x78270, 0x78320, 0x784A0, 0x6D67BC, 0x6D69B8, 0x685330, 0x685350, 0x6D6CA0, 0x6D6CB8]:
        for x in idautils.XrefsTo(a):
            f.write("%08X <- %08X %s\n" % (a, x.frm, idc.get_func_name(x.frm)))
    for a in range(0x685320, 0x6853A0, 4):
        f.write("%08X u32 %08X f32 %r\n" % (a, idc.get_wide_dword(a), __import__('struct').unpack('<f', idc.get_wide_dword(a).to_bytes(4, 'little'))[0]))
dec_list(R + r"\barrier_render.c", funcs_in(0x106B30, 0x10CCB0) + [0x10D5F0])
with open(R + r"\barrier_xrefs.txt", "w") as f:
    for fa in funcs_in(0x106B30, 0x10CCB0):
        for x in idautils.XrefsTo(fa):
            fn = idc.get_func_name(x.frm)
            if not (0x106B30 <= x.frm < 0x10CCB0):
                f.write("%08X <- %08X %s\n" % (fa, x.frm, fn))
print("ok")
