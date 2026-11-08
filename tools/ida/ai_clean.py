"""collapse assert boilerplate in research/ai_dump/*.c into ASSERT("expr") lines (ai agent, plain python).

usage: python tools/ida/ai_clean.py  -> writes research/ai_dump/clean/<file>.c
"""
import os
import re

SRC = os.path.join(os.path.dirname(__file__), "..", "..", "research", "ai_dump")
DST = os.path.join(SRC, "clean")
os.makedirs(DST, exist_ok=True)


def block_end(s, i):
    # s[i] == '{' -> index after matching '}'
    d = 0
    while i < len(s):
        c = s[i]
        if c == '"':
            i += 1
            while s[i] != '"':
                if s[i] == "\\":
                    i += 1
                i += 1
        elif c == "{":
            d += 1
        elif c == "}":
            d -= 1
            if d == 0:
                return i + 1
        i += 1
    return i


HEAD = re.compile(r"if \( off_6C18D8 != nullptr \)\s*\{")
for fn in os.listdir(SRC):
    if not fn.endswith(".c") or fn.startswith("_"):
        continue
    s = open(os.path.join(SRC, fn), encoding="utf-8").read()
    out = []
    i = 0
    while True:
        m = HEAD.search(s, i)
        if not m:
            out.append(s[i:])
            break
        b1 = block_end(s, m.end() - 1)
        body = s[m.end():b1]
        em = re.search(r'off_6C18D8\((".*?"|&unk_\w+)\s*,\s*"[^"]*",\s*(\d+)\)', body, re.S)
        rest = s[b1:]
        em2 = re.match(r"\s*else\s*\{", rest)
        if not em or not em2:
            out.append(s[i:m.end()])
            i = m.end()
            continue
        b2 = block_end(s, b1 + em2.end() - 1)
        expr = em.group(1).replace("\n", " ")
        expr = re.sub(r"\s+", " ", expr)
        if expr.startswith("&unk"):
            expr = '"false"'
        out.append(s[i:m.start()])
        out.append("ASSERT(%s); // line %s" % (expr, em.group(2)))
        i = b2
    t = "".join(out)
    # an assert guarded by a condition: "if ( cond )\n{\n ASSERT(...);\n}" -> keep, it is informative
    t = re.sub(r"\n\s*InputDevice__3DDC60\([^;]*\);", "", t)
    open(os.path.join(DST, fn), "w", encoding="utf-8").write(t)
print("ok")
