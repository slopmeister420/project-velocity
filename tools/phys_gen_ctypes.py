# physics agent: emit C struct declarations for CGlobalConfig / CLocalConfig (for IDA declare_type)
import re
rows = []
for line in open(r"D:\projects\project poland rewrite\research\phys_dump\gen_global.rs"):
    m = re.match(r"\s*pub (\w+): (.+?), // \+0x([0-9A-F]+)", line)
    if m:
        rows.append(m.groups())
def ctype(t, n):
    t = t.strip()
    m = re.match(r"\[(f32|i32); (\d+)\]", t)
    if m:
        return "%s %s[%s];" % ("float" if m.group(1) == "f32" else "int", n, m.group(2))
    if t == "f32":
        return "float %s;" % n
    if t == "i32":
        return "int %s;" % n
    if t == "Expression":
        return "void *%s;" % n
    raise Exception(t)
g = ["struct CGlobalConfig {"] + ["  " + ctype(t, n) for n, t, o in rows] + ["};"]
loc = []
src = open(r"D:\projects\project poland rewrite\crates\velocity_physics\src\c_config.rs", encoding="utf-8").read()
part = src[src.index("pub struct CLocalConfig {"):]
part = part[:part.index("\n}\n")]
for m in re.finditer(r"pub (\w+): (.+?), // \+0x([0-9A-F]+)", part):
    n, t, o = m.groups()
    t = t.strip()
    if t.startswith("[Expression; 6]"):
        loc.append("  void *%s[6];" % n)
    elif t.startswith("[[Expression; 4]; 6]"):
        loc.append("  void *%s[24];" % n)
    elif t.startswith("[[i32; 4]; 6]"):
        loc.append("  int %s[24];" % n)
    elif t.startswith("Vec<"):
        loc.append("  float *%s;" % n)
    else:
        loc.append("  " + ctype(t, n))
l = ["struct CLocalConfig {"] + loc + ["};"]
open(r"D:\projects\project poland rewrite\research\phys_dump\cconfig_types.h", "w").write("\n".join(g + l) + "\n")
print(len(rows), len(loc))
