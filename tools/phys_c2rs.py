# physics agent: translate straight-line Hex-Rays float code (the Maple-generated FastCar solvers) to Rust.
# usage: phys_c2rs.py <dump.c> <ADDR> [out.rs]
# rules: `(float)(E)` -> E computed in f32 (operands rounded to f32 first); an uncast operator is an x87
# operation -> f64; calls to the Vs* wrappers return f64 (x87 st0); stores to float storage round.
import re, sys

src = open(sys.argv[1], encoding="utf-8").read()
addr = sys.argv[2].upper().zfill(8)
start = src.index("// ==== " + addr)
end = src.find("\n// ==== ", start + 10)
body = src[start:end if end > 0 else None]
sig = body.split("\n")[1]

# ---------------- tokenizer
TOK = re.compile(r"\s*(?:(\d+\.\d*(?:e[+-]?\d+)?f?|\d+e[+-]?\d+f?|0x[0-9A-Fa-f]+|\d+)|([A-Za-z_]\w*)|(\*\(float \*\)&)|(\*\(_DWORD \*\)\()|(\*\(float \*\)\()|(\(float\))|(\(double\))|(\(int\))|(\(__int64\))|(\(unsigned int\))|([-+*/(),\[\];=<>!&|]))")

def tokenize(s):
    pos = 0
    out = []
    while pos < len(s):
        m = TOK.match(s, pos)
        if not m or m.end() == pos:
            if s[pos:].strip() == "":
                break
            raise Exception("tok at: " + s[pos:pos + 40])
        pos = m.end()
        g = m.groups()
        if g[0]:
            out.append(("num", g[0]))
        elif g[1]:
            out.append(("id", g[1]))
        elif g[2]:
            out.append(("fref", None))
        elif g[3]:
            out.append(("dwptr", None))
        elif g[4]:
            out.append(("fptr", None))
        elif g[5]:
            out.append(("cast", "f32"))
        elif g[6]:
            out.append(("cast", "f64"))
        elif g[7] or g[8] or g[9]:
            out.append(("cast", "int"))
        else:
            out.append(("op", g[10]))
    return out

class P:
    def __init__(self, toks):
        self.t = toks
        self.i = 0
    def peek(self, k=0):
        return self.t[self.i + k] if self.i + k < len(self.t) else (None, None)
    def take(self):
        x = self.t[self.i]
        self.i += 1
        return x
    def expect(self, v):
        x = self.take()
        if x[1] != v:
            raise Exception("expected %s got %s" % (v, x))

# AST: ("num", text) ("var", name, idx|None) ("bin", op, a, b) ("neg", a) ("cast", ty, a) ("call", f, args) ("ptr", base, off)
def parse_expr(p):
    return parse_add(p)

def parse_add(p):
    a = parse_mul(p)
    while p.peek()[1] in ("+", "-"):
        op = p.take()[1]
        b = parse_mul(p)
        a = ("bin", op, a, b)
    return a

def parse_mul(p):
    a = parse_unary(p)
    while p.peek()[1] in ("*", "/"):
        op = p.take()[1]
        b = parse_unary(p)
        a = ("bin", op, a, b)
    return a

def parse_unary(p):
    k, v = p.peek()
    if v == "-":
        p.take()
        return ("neg", parse_unary(p))
    if k == "cast":
        p.take()
        return ("cast", v, parse_unary(p))
    return parse_postfix(p)

def parse_postfix(p):
    k, v = p.take()
    if k == "num":
        return ("num", v)
    if k == "fref":
        _, name = p.take()
        return ("var", name, None)
    if k == "fptr":
        # *(float *)(aN + K)
        _, base = p.take()
        off = 0
        if p.peek()[1] == "+":
            p.take()
            off = int(p.take()[1], 0)
        p.expect(")")
        return ("ptr", base, off)
    if v == "(":
        e = parse_expr(p)
        p.expect(")")
        return e
    if k == "id":
        if p.peek()[1] == "(":
            p.take()
            args = []
            if p.peek()[1] != ")":
                args.append(parse_expr(p))
                while p.peek()[1] == ",":
                    p.take()
                    args.append(parse_expr(p))
            p.expect(")")
            return ("call", v, args)
        if p.peek()[1] == "[":
            p.take()
            neg = False
            if p.peek()[1] == "-":
                p.take()
                neg = True
            idx = int(p.take()[1])
            p.expect("]")
            return ("var", v, -idx if neg else idx)
        return ("var", v, None)
    if k == "op" and v == "*":
        # *aN  -> aN[0]
        _, name = p.take()
        return ("var", name, 0)
    raise Exception("bad primary %s %s" % (k, v))

FUNCS = {"VsSin": "vsmath::sin", "VsCos": "vsmath::cos", "VsSqrt": "vsmath::sqrt", "VsFabs": "vsmath::fabs",
         "VsTan": "vsmath::tan", "VsAtan": "vsmath::atan", "VsAtan2": "vsmath::atan2", "VsACos": "vsmath::acos",
         "VsExp": "vsmath::exp"}

def numlit(t):
    t = t.rstrip("f")
    if "." not in t and "e" not in t:
        t = t + ".0"
    return t

def emit(n, want, arrays):
    """returns rust expr string with type `want` ('f32' or 'f64')."""
    kind = n[0]
    if kind == "num":
        return numlit(n[1]) + ("f32" if want == "f32" else "f64")
    if kind == "var":
        name, idx = n[1], n[2]
        s = name if idx is None else "%s[%s]" % (name, idx if idx >= 0 else "NEG%d" % -idx)
        if idx is None and SCALARS.get(name) == "f64":
            return s if want == "f64" else "(%s as f32)" % s
        return s if want == "f32" else "(%s as f64)" % s
    if kind == "ptr":
        s = "%s[%d]" % (n[1], n[2] // 4)
        return s if want == "f32" else "(%s as f64)" % s
    if kind == "neg":
        return "(-%s)" % emit(n[1], want, arrays)
    if kind == "cast":
        ty = n[1]
        inner = n[2]
        if ty == "f32":
            if inner[0] == "bin":
                s = "(%s %s %s)" % (emit(inner[2], "f32", arrays), inner[1], emit(inner[3], "f32", arrays))
            else:
                s = emit(inner, "f32", arrays)
            return s if want == "f32" else "(%s as f64)" % s
        s = emit(inner, "f64", arrays)
        return s if want == "f64" else "(%s as f32)" % s
    if kind == "bin":
        s = "(%s %s %s)" % (emit(n[2], "f64", arrays), n[1], emit(n[3], "f64", arrays))
        return s if want == "f64" else "(%s as f32)" % s
    if kind == "call":
        f = FUNCS[n[1]]
        args = ", ".join(emit(a, "f32", arrays) for a in n[2])
        s = "%s(%s)" % (f, args)
        return s if want == "f64" else "(%s as f32)" % s
    raise Exception(n)

def is_single_f32(n):
    return n[0] in ("var", "ptr", "num", "cast")

lines = []
SCALARS = {}
for m in re.finditer(r"^\s*(float|double) (v\d+);", body, re.M):
    SCALARS[m.group(2)] = "f64" if m.group(1) == "double" else "f32"
flat = re.sub(r"\s+", " ", body[body.index("{") + 1:body.rindex("}")])
flat = re.sub(r"\*\((float|_DWORD) \*\)(a\d+)(?! \+)", lambda m: "*(%s *)(%s + 0)" % (m.group(1), m.group(2)), flat)
stmts = flat.split(";")
for st in stmts:
    st = st.strip()
    if not st or st.startswith("memset") or st.startswith("return") or re.match(r"^(float|int|double|_DWORD|__int64)\b", st):
        continue
    if st.startswith("//"):
        continue
    lhs, rhs = st.split(" = ", 1)
    lt = tokenize(lhs)
    if lt[0][0] == "dwptr":
        # *(_DWORD *)(aN + K) = 0
        base = lt[1][1]
        off = int(lt[3][1]) if len(lt) > 3 and lt[2][1] == "+" else 0
        val = rhs.strip()
        lines.append("    %s[%d] = %s;" % (base, off // 4, "0.0" if val == "0" else "f32::from_bits(%s as u32)" % val))
        continue
    p = P(lt)
    target = parse_postfix(p)
    rp = P(tokenize(rhs))
    e = parse_expr(rp)
    if rp.i != len(rp.t):
        raise Exception("trailing in " + rhs)
    if target[0] == "ptr":
        tname = "%s[%d]" % (target[1], target[2] // 4)
    else:
        tname = target[1] if target[2] is None else "%s[%d]" % (target[1], target[2])
    tw = SCALARS.get(tname, "f32") if target[0] == "var" and target[2] is None else "f32"
    lines.append("    %s = %s;" % (tname, emit(e, tw, None)))
joined = "\n".join(lines)
decl = ["    let mut %s: %s = 0.0;" % (n, t) for n, t in sorted(SCALARS.items()) if re.search(r"\b%s = " % n, joined)]
out = "// %s\n%s\n" % (sig.strip(), "\n".join(decl + lines))
if len(sys.argv) > 3:
    open(sys.argv[3], "w").write(out)
else:
    print(out)
