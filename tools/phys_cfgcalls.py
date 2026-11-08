# physics agent: compress a CConfig Load() decompile into one line per reader call / control statement
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
start = src.index("// ==== " + sys.argv[2])
end = src.find("\n// ==== ", start + 10)
body = src[start:end if end > 0 else None]
# join everything, then split on statements
flat = re.sub(r"\s+", " ", body)
names = {"VsConfigFileRead__1B6570": "INT", "VsConfigFileRead__1B6710": "REAL",
         "VsConfigFileRead__1B68B0": "STR", "VsConfigFileRead__1B6A20": "EXPR"}
pat = re.compile(r"(VsConfigFileRead__1B6[0-9A-F]{3})\(\s*([^;]*?)\"([^\"]+)\"\s*,([^)]*)\)")
toks = re.split(r"((?:if|while|for) \()", flat)
out = []
for m in re.finditer(r"VsConfigFileRead__1B6(?:570|710|8B0|A20)\(([^\"]*)\"([^\"]+)\"([^)]*\)?[^)]*)\)|(\bfor \([^)]*\))|(\bwhile \([^)]*\))|(\*\(_DWORD \*\)\(v\d+\[\d+\] \+ \d+\) != \d+)|(\*\(_DWORD \*\)\(v\d+\[\d+\] \+ \d+\) == \d+)", flat):
    if m.group(2):
        call = flat[m.start():m.start() + 26]
        kind = names.get(call[:24], call[:24])
        args = [a.strip() for a in m.group(1).split(",") if a.strip()]
        out.append("%s %s %s" % (kind, " ".join(args[1:]), m.group(2)))
    else:
        out.append("   ## " + m.group(0))
print("\n".join(out))
