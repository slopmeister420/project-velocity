import idc, idautils, ida_hexrays, ida_bytes, re
# audio AddTweaks registrations: decompile each into research/audio/tweak_regs_dc.c, one line per
# createSystem / addTweak call -> research/audio/tweak_regs.tsv (fn kind name var args init-bits),
# callers (two levels) -> research/audio/tweak_callers.txt
base = r"D:\projects\project poland rewrite\research\audio"
FUNCS = [0x48410, 0x48530, 0x48600, 0x4B380, 0x4CB10, 0x4D7D0, 0x4FD50]
K = {"85A90": "F", "85B40": "I", "85BF0": "E", "85CA0": "B", "86250": "S"}
NOISE = re.compile(r"__debugbreak|^\s*[{}]\s*$|off_6C18D8 != nullptr|memset\(.*0xCC|= off_6C18D8\(|^\s*$")
TW = re.compile(r"tweak__(85A90|85B40|85BF0|85CA0|86250)\((.*)\);")
SYM = re.compile(r"&?(dword|flt|byte|unk|word)_([0-9A-F]+)")
OFF = re.compile(r"\(int\)&off_([0-9A-F]+)")
def cstr(ea):
    s = idc.get_strlit_contents(ea, -1, 0)
    return s.decode("latin-1") if s else "?"
def split_args(s):
    out, depth, cur, q = [], 0, "", False
    for ch in s:
        if ch == '"':
            q = not q
        elif q:
            pass
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0 and not q:
            out.append(cur.strip()); cur = ""
        else:
            cur += ch
    out.append(cur.strip())
    return out
def callers(ea):
    out = []
    for x in idautils.CodeRefsTo(ea, 0):
        f = idc.get_func_attr(x, idc.FUNCATTR_START)
        out.append((x, f, idc.get_func_name(f)))
    return out
dc = open(base + r"\tweak_regs_dc.c", "w", encoding="utf-8")
tsv = open(base + r"\tweak_regs.tsv", "w", encoding="utf-8")
cl = open(base + r"\tweak_callers.txt", "w", encoding="utf-8")
for f in FUNCS:
    cl.write("== %x %s\n" % (f, idc.get_func_name(f)))
    for x, c, n in callers(f):
        cl.write("  %x in %x %s\n" % (x, c, n))
        for x2, c2, n2 in callers(c)[:6]:
            cl.write("      %x in %x %s\n" % (x2, c2, n2))
    dc.write("\n==== %x %s\n" % (f, idc.get_func_name(f)))
    try:
        txt = str(ida_hexrays.decompile(f))
    except Exception as ex:
        dc.write("ERR %s\n" % ex); continue
    lines = [l for l in txt.split("\n") if not NOISE.search(l)]
    dc.write("\n".join(lines) + "\n")
    stmts, cur = [], ""
    for l in lines:
        cur += " " + l.strip()
        if cur.rstrip().endswith((";", "{", "}")) or l.strip().startswith(("if", "else")):
            stmts.append(cur.strip()); cur = ""
    for s in stmts:
        m = TW.search(s)
        if not m:
            continue
        args = split_args(m.group(2))
        args = [OFF.sub(lambda q: '"%s"' % cstr(int(q.group(1), 16)), a) for a in args]
        args = [re.sub(r"\((const )?(char|int|void) \*?\)", "", a).strip() for a in args]
        k = K[m.group(1)]
        if k == "S":
            tsv.write("%x\tS\t%s\n" % (f, args[-1])); continue
        var = args[2]
        init = ""
        sm = SYM.search(var)
        if sm:
            ea = int(sm.group(2), 16)
            init = ("%08X" % ida_bytes.get_dword(ea)) if ida_bytes.is_loaded(ea) else "bss"
        tsv.write("%x\t%s\t%s\t%s\t%s\t%s\n" % (f, k, args[1], var, "|".join(args[3:]), init))
dc.close(); tsv.close(); cl.close()
print("ok")
