# frontend agent: decompile small OSD getters into research/fe_range.txt (one line each)
import idc, ida_hexrays
eas = [0xEFC90, 0xEFCD0, 0xEFCB0, 0xEFBF0, 0xEFC30, 0xEFC10, 0xEFD70, 0xEFD90, 0xEFDB0, 0xEFDD0, 0xEF770, 0xEF7A0, 0xEB140]
with open(r"D:\projects\project poland rewrite\research\fe_range.txt", "w") as o:
    for ea in eas:
        try:
            body = str(ida_hexrays.decompile(ea))
            lines = [l.strip() for l in body.split("\n") if "return" in l or "=" in l and "memset" not in l]
            o.write("%X: %s\n" % (ea, " | ".join(lines)))
        except Exception as ex:
            o.write("%X: fail %s\n" % (ea, ex))
print("ok")
