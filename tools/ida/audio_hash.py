import idc, ida_hexrays
# VsStringAnsi::Hash (ai engine event hash at PhysicsManager 0x19F36A) vs HANDY_CRC32
OUT = r"D:\projects\project poland rewrite\research\audio\vsstring_hash.c"
with open(OUT, "w", encoding="utf-8") as fo:
    ea = idc.get_name_ea_simple("VsStringAnsi::Hash")
    fo.write("// %x\n" % ea)
    try:
        c = str(ida_hexrays.decompile(ea))
    except Exception as e:
        c = "// fail %s" % e
    fo.write(c + "\n")
print("ok")
