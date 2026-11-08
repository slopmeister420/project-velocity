# parse the AddTweaks decompiles (research/ai_dump) into "path|name|kind|min|max|step" lines (hex f32 bits; ints
# as decimal) for velocity_ai's ai_tweaks test. usage: python tools/ai_tweak_table.py > out.txt
import re, struct, sys

root = r"D:\projects\project poland rewrite\research\ai_dump"


def body(path, start):
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    out = []
    for l in lines[start:]:
        if l.startswith("// ==== ") and out:
            break
        out.append(l)
    return " ".join(x.strip() for x in out)


def fbits(tok):
    tok = tok.strip()
    if tok.endswith("f"):
        tok = tok[:-1]
    if re.fullmatch(r"-?\d+", tok) and "." not in tok:
        return "%08X" % (int(tok, 0) & 0xFFFFFFFF)
    if tok.startswith("0x"):
        return "%08X" % int(tok, 16)
    return "%08X" % struct.unpack("<I", struct.pack("<f", float(tok)))[0]


NAMES = {"&off_624DF8": "TFT", "&off_626350": "0.0", "&off_626334": "0.5", "&off_626318": "1.0"}


def name(tok):
    tok = tok.strip()
    if tok.startswith("(int)"):
        tok = tok[5:]
    if tok in NAMES:
        return NAMES[tok]
    return tok.strip('"')


def emit(text):
    rows = []
    for m in re.finditer(r"tweak__(86250|85A90|85B40|85BF0|85CA0)\((.*?)\);", text):
        kind, args = m.group(1), m.group(2)
        # split on commas outside quotes / parens
        parts, depth, cur, q = [], 0, "", False
        for ch in args:
            if ch == '"':
                q = not q
            if not q and ch == "(":
                depth += 1
            if not q and ch == ")":
                depth -= 1
            if ch == "," and not q and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        parts.append(cur)
        if kind == "86250":
            rows.append("S|" + name(parts[2]))
        elif kind == "85A90":
            rows.append("F|%s|%s|%s|%s" % (name(parts[1]), fbits(parts[3]), fbits(parts[4]), fbits(parts[5])))
        elif kind == "85B40":
            rows.append("I|%s|%s|%s|%s" % (name(parts[1]), parts[3].strip(), parts[4].strip(), parts[5].strip().rstrip("u")))
        elif kind == "85BF0":
            rows.append("E|%s" % name(parts[1]))
        else:
            rows.append("B|%s" % name(parts[1]))
    return rows


w = body(root + r"\VsAIWorld.c", 309)
t = body(root + r"\VsAIPlayerTFTManager.c", 99)
for r in emit(w) + emit(t):
    print(r)
