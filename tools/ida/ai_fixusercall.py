# repair usercall prototypes in the ai regions: float returns go to st0, phantom st(n) inputs dropped,
# float-typed ecx becomes int (ai agent)
import re, idautils, idc

RANGES = [(0x12440, 0x43610), (0xE72C0, 0xE85A0), (0x16E980, 0x16F5B0)]
fixed = 0
for lo, hi in RANGES:
    for fea in idautils.Functions(lo, hi):
        t = idc.get_type(fea)
        if not t:
            continue
        m = re.match(r"^(.*?)\s*(__usercall|__userpurge)(@<[^>]+>)?\((.*)\)$", t)
        if not m:
            continue
        ret, cc, rloc, args = m.group(1).strip(), m.group(2), m.group(3) or "", m.group(4)
        parts, depth, cur = [], 0, ""
        for ch in args:
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
            if ch == "," and depth == 0:
                parts.append(cur.strip()); cur = ""
            else:
                cur += ch
        if cur.strip():
            parts.append(cur.strip())
        newparts = []
        for p in parts:
            if re.search(r"@<st\d>", p):
                continue
            if re.match(r"^float\b.*@<ecx>$", p):
                p = re.sub(r"^float", "int", p)
            newparts.append(p)
        newrloc = rloc
        if ret in ("float", "double"):
            newrloc = "@<st0>"
        if newparts == parts and newrloc == rloc:
            continue
        name = re.sub(r"[^A-Za-z0-9_]", "_", idc.get_func_name(fea))
        decl = "%s %s %s%s(%s);" % (ret, cc, name, newrloc, ", ".join(newparts))
        if idc.SetType(fea, decl):
            fixed += 1
        else:
            print("fail", hex(fea), decl)
print("fixed", fixed)
