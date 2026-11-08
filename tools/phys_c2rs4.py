# physics agent: Hex-Rays -> Rust translator (memory-space Ptr model, see vehicle_entity/cx.rs).
# usage: python tools/phys_c2rs4.py <out.rs> ADDR [ADDR...]
#   Translates the listed functions out of research/phys_dump/*.c. Each becomes `pub fn f_<addr>(cx, ...)`.
#   Signatures of every dumped function are known, so calls between translated functions are typed.
#   Calls listed in OVERRIDES go to hand-written Rust instead.
import re, sys, glob
sys.path.insert(0, r"D:\projects\project poland rewrite\tools")
from phys_cparse import tokenize, P, p_stmt

ROOT = r"D:\projects\project poland rewrite"

# ---------------- function database
FUNCS = {}   # addr -> (name, text)
STEM = {}    # addr -> dump file stem (original .cpp)
NAME2ADDR = {}
_files = sorted(glob.glob(ROOT + r"\research\phys_dump\*.c"), key=lambda f: f.endswith("Redump.c"))
for f in _files:
    t = open(f, encoding="utf-8", errors="replace").read()
    parts = re.split(r"(?m)^// ==== ([0-9A-F]{8}) (\S+)\n", t)
    redump = f.endswith("Redump.c")
    for i in range(1, len(parts), 3):
        a = parts[i].lstrip("0")
        if redump and a in FUNCS:
            # re-decompiled after a type fix: newer text, same original file
            FUNCS[a] = (FUNCS[a][0], parts[i + 2])
            continue
        if a not in FUNCS:
            FUNCS[a] = (parts[i + 1], parts[i + 2])
            STEM[a] = f.split("\\")[-1][:-2]
            NAME2ADDR[parts[i + 1]] = a
            NAME2ADDR[parts[i + 1].lstrip("_")] = a

def addr_of(n):
    a = NAME2ADDR.get(n) or NAME2ADDR.get("_" + n) or NAME2ADDR.get(n.replace("__", "::"))
    if a:
        return a
    m = re.search(r"__([0-9A-F]{4,6})$", n) or re.match(r"sub_([0-9A-F]+)$", n)
    return m.group(1).lstrip("0") if m else None

# functions IDA renamed after the dump: callers use the name in the signature line
for _a, (_n, _t) in list(FUNCS.items()):
    _head = " ".join(_t.split("{", 1)[0].split())
    _m = re.match(r"[^(]*?(?<![\w:])([A-Za-z_][\w:]*)(?:@<[\w:]+>)?\(", _head)
    if _m:
        NAME2ADDR.setdefault(_m.group(1).replace("::", "__"), _a)

NAMED = {}
try:
    for _l in open(ROOT + r"\research\phys_names_out.txt"):
        _n, _a = _l.split()
        if _a != "FFFFFFFF":
            NAMED[_n] = ("dword_" if _n.startswith("g_p") else "flt_" if len(_n) == 1 else "unk_") + _a
except FileNotFoundError:
    pass

def clean(src):
    for _n, _g in NAMED.items():
        src = re.sub(r"\b%s\b" % _n, _g, src)
    src = src.replace("operator*=", "operator_muleq").replace("operator/=", "operator_diveq").replace("operator=", "operator_assign")
    src = src.replace("operator*", "operator_mul").replace("operator-=", "operator_subeq").replace("operator+=", "operator_addeq")
    src = src.replace("operator[]", "operator_idx").replace("::~", "::dtor_")
    src = re.sub(r"(\w+)::(\w+)", r"\1__\2", src)
    src = src.replace("isnan(", "f_isnan(")
    # register annotations inside function-pointer casts (`int@<ecx>, double@<st0>`)
    src = re.sub(r"\(__usercall \*\*\)\(([^)]*)\)", lambda m: "(__thiscall **)(" + re.sub(r"@<\w+(?::\w+)?>", "", m.group(1)) + ")", src)
    return src

PTRTY = {"float": "pf", "_DWORD": "pi", "int": "pi", "unsigned int": "pi", "_BYTE": "pb", "char": "pb",
         "unsigned __int8": "pb", "bool": "pb", "void": "pv", "VehicleEntity": "pv", "__int16": "ph", "_WORD": "pw", "unsigned __int16": "pw", "WORD": "pw",
         "const char": "pb", "double": "pd", "__int64": "pq", "_QWORD": "pq"}

def ctype(ty):
    """C type string -> kind: f32 f64 i32 i64 pf pi pb pv ph void"""
    ty = re.sub(r"\s+", " ", ty.replace("const ", "").strip())
    star = ty.count("*")
    base = ty.replace("*", "").strip()
    if star >= 2:
        return "pp"
    if star == 1:
        return PTRTY.get(base, "pv")
    return {"float": "f32", "double": "f64", "__int64": "i64", "void": "void", "unsigned __int64": "i64"}.get(base, "i32")

def parse_sig(text):
    head = text.split("{", 1)[0]
    head = clean(head)
    head = re.sub(r"@<\w+(?::\w+)?>", "", head)
    m = re.match(r"\s*(.*?)\s*(?:__thiscall|__cdecl|__fastcall|__userpurge|__usercall|__stdcall)?\s*(\w+)\s*\((.*)\)\s*$", head, re.S)
    if not m:
        return None
    ret, name, ps = m.groups()
    ret = re.sub(r"__(thiscall|cdecl|fastcall|userpurge|usercall|stdcall)", "", ret).strip()
    params = []
    if ps.strip() and ps.strip() != "void":
        for p in ps.split(","):
            p = p.strip()
            mm = re.match(r"(.*?)(\w+)$", p)
            if not mm:
                params.append(("?", "x"))
                continue
            params.append((ctype(mm.group(1)), mm.group(2)))
    return ctype(ret), params

SIGS = {}
# parameters Hex-Rays invented from FPU/flag state (`double a2@<st0>`, `char a3@<efl>`): not real
# arguments; dropped from the signature (the body keeps them as locals) and from call sites that pass them
FAKE = {}
for a, (n, t) in FUNCS.items():
    s = parse_sig(t)
    if s:
        _head = t.split("{", 1)[0]
        _ps = _head[_head.index("(", _head.index(n.lstrip("_")) if n.lstrip("_") in _head else 0) + 1:_head.rindex(")")] if "(" in _head else ""
        _fake = [i for i, x in enumerate(_ps.split(",")) if re.search(r"@<(?:st\d|efl)>", x)]
        if _fake and len(_fake) < len(s[1]) + 1 and len(_ps.split(",")) == len(s[1]):
            FAKE[a] = (_fake, [s[1][i] for i in _fake])
            s = (s[0], [x for i, x in enumerate(s[1]) if i not in _fake])
        SIGS[a] = s

def int_param_is_ptr(body, name):
    if re.search(r"\((?:[\w ]+\*+)\)\s*\(?%s\b" % name, body):
        return True
    # stored into a frame slot that is then used as a base address
    for arr, k in re.findall(r"(?:LODWORD|HIDWORD)\((\w+)\[(\d+)\]\) = %s;" % name, body) + \
            re.findall(r"\b(\w+)\[(\d+)\] = \(int\)%s;" % name, body):
        if re.search(r"\(LODWORD\(%s\[%s\]\) \+ \d+\)|\*\)LODWORD\(%s\[%s\]\)|\(%s\[%s\] \+ \d+\)" % (arr, k, arr, k, arr, k), body):
            return True
    return False

def fix_sig(a):
    """int params used as addresses become void pointers"""
    ret, params = SIGS[a]
    body = clean(FUNCS[a][1])
    body = body[body.index("{"):] if "{" in body else body
    out = []
    for k, n in params:
        if k == "i32" and int_param_is_ptr(body, n):
            k = "pv"
        out.append((k, n))
    return ret, out

# ---------------- overrides: callee -> (rust template, ret kind, arg kinds)
# arg kinds: p ptr, f f32, F f64, i i32 ; template gets cx first.
OV = {
    "VsSin": ("vsmath::sin", "f64", "f"), "VsCos": ("vsmath::cos", "f64", "f"), "VsSqrt": ("vsmath::sqrt", "f64", "f"),
    "VsFabs": ("vsmath::fabs", "f64", "f"), "VsTan": ("vsmath::tan", "f64", "f"), "VsAtan": ("vsmath::atan", "f64", "f"),
    "VsAtan2": ("vsmath::atan2", "f64", "ff"), "VsACos": ("vsmath::acos", "f64", "f"), "VsAsin": ("vsmath::asin", "f64", "f"),
    "VsExp": ("vsmath::exp", "f64", "f"), "VsPow": ("vsmath::pow", "f64", "ff"), "VsAbs": ("vsmath::fabs", "f64", "f"),
    "VsClamp": ("vsmath::clamp", "f32", "fff"), "VsSinf": ("vsmath::sin", "f64", "f"), "VsSqrtf": ("vsmath::sqrt", "f64", "f"),
    "VsFge": ("vs_fge", "i32", "fff"), "VsRandUnit": ("vs_rand_unit", "f64", ""),
}
EXTRA = {}
try:
    for line in open(ROOT + r"\tools\phys_c2rs4_ov.txt", encoding="utf-8"):
        line = line.split("#")[0].strip()
        if line:
            p = line.split()
            EXTRA[p[0].upper()] = (p[1], p[2], p[3] if len(p) > 3 and p[3] != "-" else "")
except FileNotFoundError:
    pass

# ---------------- expression emission
def show(e):
    k = e[0]
    if k in ("num", "id", "str"):
        return e[1]
    if k == "idx":
        return "%s[%s]" % (show(e[1]), show(e[2]))
    if k == "call":
        return "%s(%s)" % (show(e[1]), ",".join(show(a) for a in e[2]))
    if k == "bin":
        return "(%s%s%s)" % (show(e[2]), e[1], show(e[3]))
    if k == "un":
        return "%s%s" % (e[1], show(e[2]))
    if k == "cast":
        return "(%s)%s" % (e[1], show(e[2]))
    if k == "field":
        return "%s->%s" % (show(e[1]), e[2])
    if k == "post":
        return "%s%s" % (show(e[2]), e[1])
    if k == "assign":
        return "%s%s%s" % (show(e[2]), e[1], show(e[3]))
    if k == "tern":
        return "(%s?%s:%s)" % (show(e[1]), show(e[2]), show(e[3]))
    return str(e)

PK = ("pf", "pi", "pb", "pv", "ph", "pw", "pp", "pd", "pq")
def is_ptr(t):
    return t in PK
ESZ = {"pf": 4, "pi": 4, "pb": 1, "pv": 1, "ph": 2, "pw": 2, "pp": 4, "pd": 8, "pq": 8}
EACC = {"pf": "f", "pi": "i", "pb": "b", "pv": "i", "ph": "i16", "pw": "u16", "pp": "p", "pd": "d", "pq": "q"}
ATY = {"f": "f32", "i": "i32", "b": "i32", "i16": "i32", "u16": "i32", "p": "pv", "d": "f64", "q": "i64"}
GLOB = re.compile(r"^(dword|flt|byte|word|unk|off|qword|dbl)_([0-9A-F]+)$")

class Fn:
    pass

def setup_gc():
    cc = open(ROOT + r"\crates\velocity_physics\src\c_config.rs", encoding="utf-8").read()
    g = cc[cc.index("pub struct CGlobalConfig"):cc.index("pub enum LoadResult")]
    d = {}
    for m in re.finditer(r"pub (\w+): (.+?), // \+0x([0-9A-F]+)", g):
        n, ty, o = m.groups()
        ty = ty.strip()
        d[n] = ("f32" if ty == "f32" else "i32" if ty == "i32" else "expr" if ty == "Expression" else
                "af32" if ty.startswith("[f32") else "ai32" if ty.startswith("[i32") else "?", int(o, 16))
    return d
GCT = setup_gc()

def apply_patch(addr, text, old, new):
    """source patch; Hex-Rays renumbers locals (`v29` -> `v30`) between dumps, so the old text may match
    with a consistent renaming of `vN` names, which is then applied to the new text as well"""
    if old in text:
        return text.replace(old, new)
    names = []
    def pat(n):
        if n not in names:
            names.append(n)
            return r"(?P<n%d>v\d+)" % (len(names) - 1)
        return "(?P=n%d)" % names.index(n)
    parts = re.split(r"(?<!\w)(v\d+)(?!\w)", old)
    rx = "".join(pat(x) if re.fullmatch(r"v\d+", x) else re.escape(x) for x in parts)
    m = re.search(rx, text)
    assert m, "source patch for %s does not apply: %r" % (addr, old[:60])
    ren = {n: m.group("n%d" % i) for i, n in enumerate(names)}
    new2 = re.sub(r"(?<!\w)v\d+(?!\w)", lambda x: ren.get(x.group(0), x.group(0)), new)
    return text[:m.start()] + new2 + text[m.end():]

class T:
    """translator state for one function"""
    def __init__(self, addr):
        self.addr = addr
        name, text = FUNCS[addr]
        self.name = name
        for old, new in PATCHES.get(addr, []):
            text = apply_patch(addr, text, old, new)
        for _st in re.findall(r"(\w+); // st\d", text):
            text = re.sub(r"\(\*\(void( \(__thiscall \*\*\)\([^()]*\)\)\([^;]*?\)\)\([^;]*?\);)(\s*InputDevice__3DDC30\(1\);)?(?=\s*[^;\n]*\b%s\b)" % _st,
                          lambda m: "%s = (*(float%s" % (_st, m.group(1)), text)
        text = clean(text)
        self.text = text
        self.loops = []
        self.nloop = 0
        ret, params = fix_sig(addr)
        self.ret = ret
        self.params = params
        self.LT = {}
        self.frame = None    # name of the frame array
        self.frame_size = 0
        self.arrays = {}     # other arrays: name -> (kind, n)
        body = text[text.index("{"):]
        decls = []
        for m in re.finditer(r"^\s*([A-Za-z_][\w ]*?)\s*(\**)\s*(\w+)(\[(\d+)\])?;\s*//", body, re.M):
            decls.append(m.groups())
        for ty, star, n, _, arr in decls:
            k = ctype(ty + star)
            if arr:
                self.arrays[n] = (k, int(arr))
            else:
                self.LT[n] = k
        for k, n in params:
            self.LT[n] = k
        for k, n in FAKE.get(addr, ((), ()))[1]:
            self.LT[n] = k
        # the debug frame: the array filled with 0xCC
        m = re.search(r"memset\((\w+), (?:0xCCu|204), sizeof\(\1\)\)", body)
        if m and m.group(1) in self.arrays:
            self.frame = m.group(1)
        self.abase = {}
        off = 0
        for nm in ([self.frame] if self.frame else []) + [x for x in self.arrays if x != self.frame]:
            k, n = self.arrays[nm]
            self.abase[nm] = off
            off += n * (8 if k in ("i64", "f64") else 4)
        self.frame_size = off
        for n in re.findall(r"\*\(float \*\)&(\w+)", body):
            if n in self.LT and self.LT[n] == "i32":
                self.LT[n] = "f32bits"
        fr = self.frame or "NOFRAME"
        self.ptrslots = set(int(x) for x in re.findall(r"\*\)\s*\(LODWORD\(%s\[(\d+)\]\) \+ \d+\)" % fr, body))
        self.ptrslots |= set(int(x) for x in re.findall(r"\(\w[\w ]*\*\)\s*LODWORD\(%s\[(\d+)\]\)" % fr, body))
        self.ptrslots |= set(int(x or y) for x, y in re.findall(r"\(\w[\w ]*\*\)\s*(?:\(%s\[(\d+)\] \+ \d+\)|%s\[(\d+)\])" % (((self.frame or "NOFRAME"),) * 2), body) if self.frame and self.arrays[self.frame][0] == "i32")
        self.calls = set()
        self.tmp = 0
        self.body = body

    # ---- helpers
    def fidx(self, e):
        """element of a frame array with a constant index -> 4-byte slot number"""
        if e[0] == "idx" and e[1][0] == "id" and e[1][1] in self.arrays and e[2][0] == "num":
            k, n = self.arrays[e[1][1]]
            if k in ("i64", "f64"):
                return None
            return self.abase[e[1][1]] // 4 + int(e[2][1])
        return None

    def slot_kind(self, slot):
        for nm, b in self.abase.items():
            k, n = self.arrays[nm]
            if b // 4 <= slot < b // 4 + n:
                return k
        return "f32"

    def lodw(self, e):
        if e[0] == "call" and e[1][0] == "id" and e[1][1] in ("LODWORD", "SLODWORD") and len(e[2]) == 1:
            return self.fidx(e[2][0])
        return None

    def frame_elem(self, slot=None):
        if slot is not None:
            return self.slot_kind(slot)
        return self.arrays[self.frame][0] if self.frame else None

    def ptrish(self, e):
        k = e[0]
        n = self.lodw(e)
        if n is not None:
            return n in self.ptrslots
        n = self.fidx(e)
        if n is not None:
            return n in self.ptrslots
        if k == "id":
            return is_ptr(self.LT.get(e[1], "")) or e[1] in self.arrays or e[1] == "GC" or bool(GLOB.match(e[1])) and e[1].startswith(("off_", "unk_"))
        if k == "un" and e[1] == "&":
            return True
        if k == "un" and e[1] == "*":
            x = e[2]
            if x[0] == "cast" and EACC.get(ctype(x[1]), "i") in ("f", "b", "i16", "u16", "d"):
                return False
            if x[0] != "cast" and EACC.get(self.emit_ptr(x)[1], "i") in ("f", "b", "i16", "u16", "d"):
                return False
            return True
        if k == "field":
            if show(e[1]) == "GC":
                return GCT[e[2]][0] in ("expr", "af32", "ai32")
            return show(e[1]) == "this"
        if k == "cast":
            return e[1].replace(" ", "").endswith("*") or self.ptrish(e[2])
        if k == "call":
            r = self.call_ret(e)
            return is_ptr(r)
        if k == "bin" and e[1] in "+-":
            return self.ptrish(e[2]) or (e[1] == "+" and self.ptrish(e[3]))
        if k == "idx":
            return False
        return False

    def call_ret(self, e):
        f = e[1]
        if f[0] != "id":
            return "?"
        n = f[1]
        if n in OV:
            return OV[n][1]
        a = addr_of(n)
        if a and a.upper() in EXTRA:
            return EXTRA[a.upper()][1]
        if n.upper() in EXTRA:
            return EXTRA[n.upper()][1]
        if a and a in SIGS:
            return SIGS[a][0]
        return "?"

    def emit_ptr(self, e):
        """-> (rust, ptrkind)"""
        k = e[0]
        if k == "un" and e[1] == "*" and not (e[2][0] == "cast" and EACC.get(ctype(e[2][1]), "i") in ("f", "b", "i16", "u16")):
            inner = e[2]
            if inner[0] == "cast":
                p = self.emit_ptr(inner[2])[0]
                # `*(float **)x` is a float pointer
                _b = re.sub(r"\s+", " ", inner[1].replace("const ", "").replace("*", "").strip())
                _n = inner[1].count("*")
                # `**(float ***)x`: deeper pointers remember their star count down to the pointee
                return hoist("ptr_at", p), (PTRTY.get(_b, "pv") if _n == 2 else "pp|%d|%s" % (_n - 1, _b) if _n > 2 else "pv")
            p, pt = self.emit_ptr(inner)
            if pt.startswith("pp|"):
                _, _n, _b = pt.split("|")
                return hoist("ptr_at", p), (PTRTY.get(_b, "pv") if int(_n) == 2 else "pp|%d|%s" % (int(_n) - 1, _b))
            return hoist("ptr_at", p), "pv"
        if k == "cast" and e[2][0] == "id" and GLOB.match(e[2][1]) and (is_ptr(ctype(e[1])) or ctype(e[1]) == "i32"):
            # `(int)dword_752638` as an address: a global array decays to its address (a scalar would
            # need no cast)
            return "Ptr::glob(0x%s)" % GLOB.match(e[2][1]).group(2), ctype(e[1]) if is_ptr(ctype(e[1])) else "pv"
        if k == "cast":
            ck = ctype(e[1])
            if is_ptr(ck):
                if self.ptrish(e[2]):
                    s, t = self.emit_ptr(e[2])
                    return s, ck
                s, t = self.emit(e[2])
                if t in ("i32", "ilit") and s == "0":
                    return "Ptr::NULL", ck
                return "Ptr::raw(%s)" % conv(s, t, "i32"), ck
            if ck in ("i32",) and self.ptrish(e[2]):
                s, t = self.emit_ptr(e[2])
                return s, "pv"
        if k == "bin" and e[1] in "+-":
            if self.ptrish(e[2]):
                a, ta = self.emit_ptr(e[2])
                i = self.emit_int(e[3])
                if e[1] == "-":
                    i = "-(%s)" % i
                return self.padd(a, ta, i), ta
            if e[1] == "+" and self.ptrish(e[3]):
                a, ta = self.emit_ptr(e[3])
                return self.padd(a, ta, self.emit_int(e[2])), ta
        if k == "field" and show(e[1]) == "GC":
            ty, off = GCT[e[2]]
            return ("Ptr::gexpr(%d)" if ty == "expr" else "Ptr::gc(%d)") % off, "pf"
        if k == "id" and GLOB.match(e[1]) and e[1].startswith(("unk_", "off_", "a")):
            return "Ptr::glob(0x%s)" % GLOB.match(e[1]).group(2), "pf"
        s, t = self.emit(e)
        if is_ptr(t):
            return s, t
        if t in ("i32", "ilit") and s == "0":
            return "Ptr::NULL", "pv"
        if k == "str":
            return "Ptr::NULL /*str*/", "pv"
        return "Ptr::raw(%s)" % conv(s, t, "i32"), "pv"

    def padd(self, a, ta, i):
        sz = ESZ.get(ta, 1)
        if re.match(r"^-?\d+$", i):
            return "%s.add(%d)" % (a, int(i) * sz)
        return "%s.add(%s)" % (a, i) if sz == 1 else "%s.add(%d * (%s))" % (a, sz, i)

    def deref(self, e):
        """-> (accessor, ptr rust) for an lvalue/rvalue memory access"""
        if e[0] == "un" and e[1] == "*":
            inner = e[2]
            if inner[0] == "cast":
                ck = ctype(inner[1])
                x = inner[2]
                if x[0] == "un" and x[1] == "&" and x[2][0] == "id" and x[2][1] in self.LT:
                    self.lvar_ck = ck
                    return "lvar", x[2][1]
                p, pt = self.emit_ptr(x)
                if ck == "pp":
                    # `*(float **)x` loads a float pointer: keep the pointee type
                    _b = re.sub(r"\s+", " ", inner[1].replace("const ", "").replace("*", "").strip())
                    self.pp_sub = PTRTY.get(_b, "pv") if inner[1].count("*") == 2 else "pv"
                return EACC.get(ck, "i"), p
            p, pt = self.emit_ptr(inner)
            return EACC.get(pt, "i"), p
        if e[0] == "idx" and self.fidx(e) is None:
            if e[1][0] == "id" and e[1][1] in self.arrays:
                k, n = self.arrays[e[1][1]]
                i = self.emit_int(e[2])
                return ("i" if k == "i32" else "f"), "cx.lptr(%d).add(4 * (%s))" % (self.abase[e[1][1]], i)
            if e[1][0] == "id" and GLOB.match(e[1][1]):
                g = GLOB.match(e[1][1])
                sz = {"dword": 4, "flt": 4, "byte": 1, "word": 2, "dbl": 8}.get(g.group(1), 4)
                return {"dword": "i", "flt": "f", "byte": "b", "word": "i16", "dbl": "d"}.get(g.group(1), "i"), \
                    "Ptr::glob(0x%s).add(%d * (%s))" % (g.group(2), sz, self.emit_int(e[2]))
            if e[1][0] == "field" and show(e[1][1]) == "GC":
                return None
            b, bt = self.emit_ptr(e[1])
            if bt == "pv" and e[1][0] == "un" and e[1][1] == "*" and e[1][2][0] != "cast" and self.emit_ptr(e[1][2])[1] == "pp":
                # `(*(this + 25))[9]` with `_DWORD **this`: a subscripted pointee is a typed pointer, 4-byte elements
                # (0x001E4AE0 read the body at +9 instead of +36)
                bt = "pi"
            i = self.emit_int(e[2])
            return EACC.get(bt, "f"), self.padd(b, bt, i)
        return None

    def emit_int(self, e):
        s, t = self.emit(e)
        return conv(s, t, "i32")

    def emit_val(self, e, want):
        s, t = self.emit(e)
        if want == "bool" and t == "i64" and self.is_call64(e):
            return "(%s as i32 != 0)" % s
        return conv(s, t, want)

    def is_call64(self, e):
        """a direct call of a function Hex-Rays typed `__int64`: as a truth value MSVC tests eax only
        (`test eax, eax`, checked at every such call site, e.g. 0x0037885E -> 0x0037B860; edx holds garbage)"""
        return e[0] == "call" and not (e[1][0] == "id" and re.match(r"^(S?[LH]I?DWORD|[LH]OBYTE|BYTE\d|[LH]I?WORD|__PAIR64__)$", e[1][1]))

    def frame_read(self, n, as_int):
        fe = self.frame_elem(n)
        if fe in ("i32",):
            return ("cx.li(%d)" % (4 * n), "i32") if as_int else ("f32::from_bits(cx.li(%d) as u32)" % (4 * n), "f32")
        if fe == "f64":
            return ("cx.ld(%d)" % (8 * n), "f64")
        return ("cx.li(%d)" % (4 * n), "i32") if as_int else ("cx.lf(%d)" % (4 * n), "f32")

    def emit(self, e):
        k = e[0]
        n = self.lodw(e)
        if n is not None:
            if n in self.ptrslots:
                return "cx.lpp(%d)" % n, "pv"
            return self.frame_read(n, True)
        n = self.fidx(e)
        if n is not None:
            if n in self.ptrslots:
                return "cx.lpp(%d)" % n, "pv"
            return self.frame_read(n, self.frame_elem(n) == "i32")
        if k == "un" and e[1] == "&":
            x = e[2]
            n = self.fidx(x)
            if n is not None:
                return "cx.lptr(%d)" % (4 * n), ("pi" if self.frame_elem(n) == "i32" else "pf")
            if x[0] == "id" and GLOB.match(x[1]):
                return "Ptr::glob(0x%s)" % GLOB.match(x[1]).group(2), "pv"
            if x[0] == "id" and x[1] in ("savedregs", "retaddr"):
                return "Ptr::NULL", "pv"
            if x[0] == "field" and show(x[1]) == "GC":
                return "Ptr::gc(%d)" % GCT[x[2]][1], "pf"
            if x[0] == "field" and show(x[1]) == "this":
                return "%s.add(%d)" % ("this", FIELDS.get(x[2], 0)), "pv"
            if x[0] == "idx" and x[1][0] == "id" and x[1][1] in self.arrays:
                i = self.emit_int(x[2])
                return "cx.lptr(%d).add(4 * (%s))" % (self.abase[x[1][1]], i), "pf"
            if x[0] == "id" and x[1] in self.arrays:
                return "cx.lptr(%d)" % self.abase[x[1]], "pf"
            if x[0] == "idx" and x[1][0] == "id" and GLOB.match(x[1][1]):
                g = GLOB.match(x[1][1])
                sz = {"dword": 4, "flt": 4, "byte": 1, "word": 2, "dbl": 8}.get(g.group(1), 4)
                return "Ptr::glob(0x%s).add(%d * (%s))" % (g.group(2), sz, self.emit_int(x[2])), "pf"
            if x[0] == "idx" and self.ptrish(x[1]):
                b, bt = self.emit_ptr(x[1])
                return self.padd(b, bt, self.emit_int(x[2])), bt
            if x[0] == "id" and self.LT.get(x[1]) in ("i64", "f64"):
                return "Ptr::NULL", "pv"
            return "/*TODO & %s*/" % show(x), "?"
        mh = re.fullmatch(r"\*\(\(float\*\)&(v\d+)\+1\)", show(e).replace(" ", ""))
        if mh and self.LT.get(mh.group(1)) == "i64":
            return "f32::from_bits((%s >> 32) as u32)" % mh.group(1), "f32"
        d = self.deref(e)
        if d:
            a, p = d
            if a == "lvar":
                t = self.LT[p]
                ck, self.lvar_ck = getattr(self, "lvar_ck", None), None
                if t == "i64" and ck == "pf":
                    # `*(float *)&v64`: the low dword as a float
                    return "f32::from_bits(%s as u32)" % p, "f32"
                if t == "i64" and ck == "pi":
                    return "(%s as i32)" % p, "i32"
                return (p, "f32") if t == "f32bits" or t == "f32" else (p, t)
            if a == "p":
                sub, self.pp_sub = getattr(self, "pp_sub", "pv"), "pv"
                return hoist("ptr_at", p), sub
            return hoist(a, p), ATY[a]
        if k == "num":
            return numlit(e[1])
        if k == "str":
            return e[1], "str"
        if k == "id":
            nm = e[1]
            if nm in self.LT:
                t = self.LT[nm]
                return nm, ("f32" if t == "f32bits" else t)
            if nm in self.arrays:
                return "cx.lptr(%d)" % self.abase[nm], ("pi" if self.arrays[nm][0] == "i32" else "pf")
            if nm == "GC":
                return "Ptr::gc(0)", "pv"
            if nm == "NAN":
                # Hex-Rays prints the -1 sentinel stored in a float slot as NAN
                return "f32::from_bits(0xFFFF_FFFF)", "f32"
            if nm in ("true", "false"):
                return nm, "bool"
            if nm == "nullptr":
                return "Ptr::NULL", "pv"
            m = GLOB.match(nm)
            if m:
                a = {"dword": "i", "flt": "f", "byte": "b", "word": "i16", "dbl": "d"}.get(m.group(1), "i")
                return "cx.%s(Ptr::glob(0x%s))" % (a, m.group(2)), ATY[a]
            fa = addr_of(nm)
            if not fa and re.match(r"^[A-Z]\w*__\w+$", nm):
                # a class method used as a value (constructor pointer); not called through here
                return "Ptr::func(0)", "pv"
            if fa:
                # a function used as a value (callback / constructor pointer)
                return "Ptr::func(0x%s)" % fa, "pv"
            return nm, "?"
        if k == "field":
            if show(e[1]) == "GC":
                ty, off = GCT[e[2]]
                if ty == "expr":
                    return "Ptr::gexpr(%d)" % off, "pv"
                return "cx.gc.%s" % e[2], ty
            if show(e[1]) == "this" and e[2] in FIELDS:
                if e[2] == "LC":
                    return "cx.ptr_at(this.add(%d))" % FIELDS[e[2]], "pv"
                # pads/structs decay to their address
                return "this.add(%d)" % FIELDS[e[2]], "pv"
            return "/*TODO field %s*/" % show(e), "?"
        if k == "idx":
            if e[1][0] == "field" and show(e[1][1]) == "GC":
                ty, off = GCT[e[1][2]]
                return "cx.gc.%s[%s as usize]" % (e[1][2], self.emit_int(e[2])), ty[1:]
            if e[1][0] == "id" and (e[1][1] in self.arrays or GLOB.match(e[1][1])):
                a, p = self.deref(e)
                return hoist(a, p), ATY[a]
            return "/*TODO idx %s*/" % show(e), "?"
        if k == "cast":
            ty = e[1].replace(" ", "")
            inner = e[2]
            ck = ctype(e[1])
            if ty == "float":
                if inner[0] == "bin" and inner[1] in "+-*/":
                    a = self.emit_val(inner[2], "f32")
                    b = self.emit_val(inner[3], "f32")
                    return "(%s %s %s)" % (a, inner[1], b), "f32"
                if inner[0] == "un" and inner[1] == "-":
                    return "(-%s)" % self.emit_val(inner[2], "f32"), "f32"
                s, t = self.emit(inner)
                return conv(s, t, "f32"), "f32"
            if ty == "double":
                return self.emit_val(inner, "f64"), "f64"
            if is_ptr(ck):
                return self.emit_ptr(e)
            if ck in ("i32", "i64"):
                if inner[0] == "str":
                    return "0 /*str*/", "i32"
                if ty in ("char", "_BYTE", "unsigned__int8", "bool") and inner[0] == "id" and is_ptr(self.LT.get(inner[1])):
                    # low byte of a pointer-typed local holding a bool (`LOBYTE(v1) = f(); if ((_BYTE)v1)`, 0x001E4AE0)
                    return "(%s.bits() as u8 as i32)" % inner[1], "i32"
                if self.ptrish(inner):
                    return self.emit_ptr(inner)
                s, t = self.emit(inner)
                if ty in ("char", "_BYTE", "unsigned__int8", "bool"):
                    if t in ("f32", "f64"):
                        return "(%s as i32 as i8 as i32)" % s, "i32"
                    return "(%s as u8 as i32)" % conv(s, t, "i32") if ty in ("_BYTE", "unsigned__int8", "bool") else "(%s as i8 as i32)" % conv(s, t, "i32"), "i32"
                if ty in ("__int16",):
                    return "(%s as i16 as i32)" % conv(s, t, "i32"), "i32"
                if ty == "unsigned__int16":
                    return "(%s as u16 as i32)" % conv(s, t, "i32"), "i32"
                if ck == "i64":
                    if t == "i32" and ty.startswith("unsigned"):
                        return "((%s) as i32 as u32 as i64)" % s, "i64"
                    return conv(s, t, "i64"), "i64"
                return conv(s, t, "i32"), "i32"
            return self.emit(inner)
        if k == "un":
            op = e[1]
            if op == "-":
                s, t = self.emit(e[2])
                if t in ("lit", "ilit"):
                    return "-" + s, t
                if t == "i32":
                    return "%s.wrapping_neg()" % s, t
                return "(-%s)" % s, t
            if op == "!":
                return "!%s" % self.emit_val(e[2], "bool"), "bool"
            if op in ("++", "--"):
                return self.incdec(e[2], "pre", op)
            if op == "~":
                return "!%s" % self.emit_int(e[2]), "i32"
            return "/*TODO %s*/" % show(e), "?"
        if k == "post":
            return self.incdec(e[2], "post", e[1])
        if k == "bin":
            op = e[1]
            if op in ("&&", "||"):
                return "(%s %s %s)" % (self.emit_val(e[2], "bool"), op, self.emit_val(e[3], "bool")), "bool"
            if op in "+-" and (self.ptrish(e[2]) or (op == "+" and self.ptrish(e[3]))):
                pa, pt = (self.ptrish(e[2]), None)
                # pointer arithmetic only when the pointer side really is a pointer value
                s, t = self.emit(e[2])
                if is_ptr(t):
                    return self.emit_ptr(e)
                s2, t2 = self.emit(e[3])
                if is_ptr(t2) and op == "+":
                    return self.emit_ptr(e)
            a, ta = self.emit(e[2])
            b, tb = self.emit(e[3])
            if op in ("<", ">", "<=", ">=", "==", "!="):
                if is_ptr(ta) or is_ptr(tb):
                    return "(%s %s %s)" % (a + ".bits()" if is_ptr(ta) else conv(a, ta, "i32"), op,
                                           b + ".bits()" if is_ptr(tb) else conv(b, tb, "i32")), "bool"
                for w in ("f64", "f32"):
                    if w in (ta, tb):
                        return "(%s %s %s)" % (conv(a, ta, w), op, conv(b, tb, w)), "bool"
                if "lit" in (ta, tb):
                    return "(%s %s %s)" % (conv(a, ta, "f64"), op, conv(b, tb, "f64")), "bool"
                if op in ("==", "!=") and ((ta == "i64" and self.is_call64(e[2]) and e[3][0] == "num" and tb == "ilit")
                                           or (tb == "i64" and self.is_call64(e[3]) and e[2][0] == "num" and ta == "ilit")):
                    # `f() != 0` on a fake __int64 return: the low dword (see is_call64)
                    return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "bool"
                if "i64" in (ta, tb):
                    return "(%s %s %s)" % (conv(a, ta, "i64"), op, conv(b, tb, "i64")), "bool"
                return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "bool"
            ints = ("i32", "ilit", "bool")
            if ta in ints and tb in ints:
                if ta == "ilit" and tb == "ilit":
                    return "(%s %s %s)" % (a, op, b), "ilit"
                if op in ("+", "-", "*"):
                    fn = {"+": "wrapping_add", "-": "wrapping_sub", "*": "wrapping_mul"}[op]
                    aa = conv(a, ta, "i32") if ta != "ilit" else "(%s_i32)" % a
                    return "%s.%s(%s)" % (aa, fn, conv(b, tb, "i32")), "i32"
                if op in ("<<", ">>") :
                    return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "i32"
                return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "i32"
            if "i64" in (ta, tb) and ta not in ("f32", "f64", "lit") and tb not in ("f32", "f64", "lit"):
                return "(%s %s %s)" % (conv(a, ta, "i64"), op, conv(b, tb, "i64")), "i64"
            if op in ("&", "|", "^", "%", "<<", ">>"):
                return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "i32"
            return "(%s %s %s)" % (conv(a, ta, "f64"), op, conv(b, tb, "f64")), "f64"
        if k == "call":
            return self.emit_call(e)
        if k == "tern":
            a, ta = self.emit(e[2])
            b, tb = self.emit(e[3])
            if is_ptr(ta) or is_ptr(tb):
                w = "pv"
            elif ta == tb and ta not in ("lit", "ilit"):
                w = ta
            else:
                w = "f64" if "f64" in (ta, tb) else "f32" if "f32" in (ta, tb) else "f64" if "lit" in (ta, tb) else "i32"
            return "(if %s { %s } else { %s })" % (self.emit_val(e[1], "bool"), conv(a, ta, w), conv(b, tb, w)), w
        if k == "assign":
            l = e[2]
            if e[1] == "=" and l[0] == "id" and l[1] in self.LT:
                t = self.LT[l[1]]
                return "{ %s %s }" % (self.emit_assign(e[1], e[2], e[3]).strip(), l[1]), ("f32" if t == "f32bits" else t)
            return "{ %s }" % self.emit_assign(e[1], e[2], e[3]).strip(), "void"
        if k == "comma":
            a = self.emit(e[1])[0]
            b, tb = self.emit(e[2])
            return "{ let _ = %s; %s }" % (a, b), tb
        return "/*TODO %s*/" % show(e), "?"

    def incdec(self, x, pp, op):
        inc = "inc" if op == "++" else "dec"
        if x[0] == "call" and x[1][0] == "id" and x[1][1] in ("HIBYTE", "LOBYTE") and self.fidx(x[2][0]) is not None:
            off = 4 * self.fidx(x[2][0])
            sh = 24 if x[1][1] == "HIBYTE" else 0
            d = "1" if op == "++" else "255"
            return ("{ let o = cx.li(%d); let b = (((o >> %d) & 255) + %s) & 255; cx.set_li(%d, (o & !(255 << %d)) | (b << %d)); b }"
                    % (off, sh, d, off, sh, sh)), "i32"
        d = self.deref(x)
        if d and d[0] in ("i", "b", "f"):
            return "cx.%s%s_%s(%s)" % (pp, inc, d[0], d[1]), ATY[d[0]]
        if d and d[0] in ("u16", "i16"):
            # 16-bit memory ++/--: wraps at 16 bits, the value read back is the stored word
            dd = "1" if op == "++" else "-1"
            rd = "cx.%s(p)" % d[0]
            if pp == "pre":
                return "{ let p = %s; let v = %s.wrapping_add(%s); cx.set_%s(p, v); %s }" % (d[1], rd, dd, d[0], rd), "i32"
            return "{ let p = %s; let v = %s; cx.set_%s(p, v.wrapping_add(%s)); v }" % (d[1], rd, d[0], dd), "i32"
        n = self.lodw(x)
        if n is None:
            n = self.fidx(x)
            n = n if n is not None and self.frame_elem(n) == "i32" else None
        if n is not None:
            return "cx.%s%s_i(cx.lptr(%d))" % (pp, inc, 4 * n), "i32"
        nm = show(x)
        if x[0] == "id" and GLOB.match(nm):
            g = GLOB.match(nm)
            a = {"dword": "i", "flt": "f", "byte": "b"}.get(g.group(1), "i")
            return "cx.%s%s_%s(Ptr::glob(0x%s))" % (pp, inc, a, g.group(2)), ATY[a]
        t = self.LT.get(nm, "i32")
        one = "1" if t in ("i32", "i64") else "1.0"
        if pp == "pre":
            return "{ %s %s= %s; %s }" % (nm, "+" if op == "++" else "-", one, nm), t
        return "{ let t = %s; %s %s= %s; t }" % (nm, nm, "+" if op == "++" else "-", one), t

    def emit_call(self, e):
        f = e[1]
        if f[0] == "id":
            n = f[1]
            if n in ("LODWORD", "SLODWORD"):
                a, t = self.emit(e[2][0])
                if t == "f32":
                    return "(%s.to_bits() as i32)" % a, "i32"
                if t == "f64":
                    return "((%s as f32).to_bits() as i32)" % a, "i32"
                if t == "i64":
                    return "(%s as i32)" % a, "i32"
                return conv(a, t, "i32"), "i32"
            if n == "HIDWORD":
                a, t = self.emit(e[2][0])
                return "((%s >> 32) as i32)" % conv(a, t, "i64"), "i32"
            if n == "signbit":
                return "(%s.is_sign_negative() as i32)" % self.emit_val(e[2][0], "f32"), "i32"
            if n in ("LOBYTE", "BYTE1", "BYTE2", "BYTE3", "HIBYTE", "LOWORD", "HIWORD", "SLOWORD") and self.fidx(e[2][0]) is not None:
                sh, w = {"LOBYTE": (0, 0xFF), "BYTE1": (8, 0xFF), "BYTE2": (16, 0xFF), "BYTE3": (24, 0xFF), "HIBYTE": (24, 0xFF),
                         "LOWORD": (0, 0xFFFF), "SLOWORD": (0, 0xFFFF), "HIWORD": (16, 0xFFFF)}[n]
                return "((cx.li(%d) >> %d) & %d)" % (4 * self.fidx(e[2][0]), sh, w), "i32"
            if n in ("LOBYTE", "BYTE1", "BYTE2", "HIBYTE", "LOWORD", "HIWORD", "SHIDWORD", "SLOWORD") and e[2][0][0] == "id":
                sh, w = {"LOBYTE": (0, 0xFF), "BYTE1": (8, 0xFF), "BYTE2": (16, 0xFF), "HIBYTE": (24, 0xFF), "LOWORD": (0, 0xFFFF),
                         "SLOWORD": (0, 0xFFFF), "HIWORD": (16, 0xFFFF), "SHIDWORD": (32, 0xFFFFFFFF)}[n]
                a, t = self.emit(e[2][0])
                return "((%s >> %d) & %d) as i32" % (conv(a, t, "i64"), sh, w), "i32"
            if n in ("COERCE_FLOAT",):
                return "f32::from_bits(%s as u32)" % self.emit_int(e[2][0]), "f32"
            if n in ("memset",):
                p = self.emit_ptr(e[2][0])[0]
                return "cx.memset(%s, %s, %s)" % (p, self.emit_int(e[2][1]), self.emit_int(e[2][2])), "void"
            if n in ("qmemcpy", "memcpy"):
                return "cx.memcpy(%s, %s, %s)" % (self.emit_ptr(e[2][0])[0], self.emit_ptr(e[2][1])[0], self.emit_int(e[2][2])), "void"
            a = addr_of(n)
            if n in OV:
                name, rt, ak = OV[n]
                cxa = not name.startswith(("vsmath::", "i32::"))
            elif (a and a.upper() in EXTRA) or n.upper() in EXTRA:
                name, rt, ak = EXTRA[a.upper() if a and a.upper() in EXTRA else n.upper()]
                cxa = not name.startswith(("vsmath::", "i32::"))
            elif a and a in SIGS:
                rt, ps = fix_sig(a)
                ak = "".join({"f32": "f", "f64": "F", "i32": "i", "i64": "q"}.get(k, "p") for k, _ in ps)
                name = "f_" + a.lower()
                cxa = True
                self.calls.add(a)
                if a in FAKE and len(e[2]) == len(ps) + len(FAKE[a][0]):
                    e = (e[0], e[1], [x for i, x in enumerate(e[2]) if i not in FAKE[a][0]])
            else:
                name, rt, ak = "h_" + n.lower(), "?", None
                cxa = True
                self.calls.add(n)
            args = []
            for i, x in enumerate(e[2]):
                kind = ak[i] if ak and i < len(ak) else None
                if kind == "p":
                    if x[0] == "id" and GLOB.match(x[1]):
                        # a global array passed by name decays to its address
                        args.append("Ptr::glob(0x%s)" % GLOB.match(x[1]).group(2))
                    else:
                        args.append(self.emit_ptr(x)[0])
                elif kind == "f":
                    args.append(self.emit_val(x, "f32"))
                elif kind == "F":
                    args.append(self.emit_val(x, "f64"))
                elif kind == "i":
                    s, t = self.emit(x)
                    # a float pushed into a parameter the decompiler typed `int` keeps its bits
                    args.append(s + ".bits()" if is_ptr(t) else "0 /*str*/" if t == "str" else
                                "(%s.to_bits() as i32)" % s if t == "f32" else conv(s, t, "i32"))
                elif kind == "q":
                    args.append(self.emit_val(x, "i64"))
                else:
                    s, t = self.emit(x)
                    args.append(conv(s, t, "f32") if t == "lit" else s)
            if name == "crate::ov_unmodelled!":
                k = {"pv": "p", "i32": "i", "f32": "f", "f64": "d", "void": "v"}.get(rt, "i")
                lets = "".join("let _ = %s; " % x for x in args)
                return "{ %sunmodelled_%s(\"%s\") }" % (lets, k, n), rt
            if not cxa:
                return "%s(%s)" % (name, ", ".join(args)), rt
            if any("cx" in x for x in args):
                lets = "".join("let q%d_ = %s; " % (i, x) for i, x in enumerate(args))
                return "{ %s%s(cx%s) }" % (lets, name, "".join(", q%d_" % i for i in range(len(args)))), rt
            return "%s(cx%s)" % (name, "".join(", " + x for x in args)), rt
        # virtual call (*(T **)(*(_DWORD *)X + K))(X, args)
        if f[0] == "un" and f[1] == "*":
            x = f[2][2] if f[2][0] == "cast" else f[2]
            slot = int(x[3][1]) if x[0] == "bin" and x[1] == "+" and x[3][0] == "num" else 0
            args = [self.emit_ptr(e[2][0])[0]] + [self.emit(x)[0] for x in e[2][1:]]
            lets = "".join("let q%d_ = %s; " % (i, x) for i, x in enumerate(args))
            _m = re.search(r"vfn_(\w+)", show(f))
            rty = {"void": "void", "float": "f32", "double": "f64", "int": "i32", "char": "i32", "_DWORD": "i32",
                   "int64": "i64", "ptr": "pv"}.get(_m.group(1), "void") if _m else "void"
            call = "{ %svcall(cx, %d, q0_, &[%s]) }" % (lets, slot, ", ".join("Varg::from(q%d_)" % i for i in range(1, len(args))))
            if rty == "f32":
                return call + ".f()", "f32"
            if rty == "f64":
                return "(" + call + ".f() as f64)", "f64"
            if rty in ("i32", "i64"):
                return call + ".i()", "i32"
            if is_ptr(rty):
                return call + ".p()", "pv"
            return "{ let _ = " + call + "; }", "void"
        return "/*TODO call %s*/" % show(e), "?"

    def emit_assign(self, op, l, r):
        sl = show(l)
        _mhr = re.fullmatch(r"\*\(\(float\*\)&(v\d+)\+1\)", show(r).replace(" ", ""))
        if op == "=" and _mhr and self.LT.get(_mhr.group(1)) == "i64" and self.fidx(l) is not None:
            # a frame slot loaded from the high dword of a 64-bit local (bits kept: often a pointer)
            return "{ let t = (%s >> 32) as i32; cx.set_li(%d, t); }" % (_mhr.group(1), 4 * self.fidx(l))
        if op == "=" and r[0] == "id" and r[1].startswith("off_") and GLOB.match(r[1]) and sl.replace(" ", "") in ("*this", "*(_DWORD*)this", "*(_DWORD**)this"):
            # constructor storing its vtable: the table's address
            return "{ let p = this; let t = Ptr::glob(0x%s); cx.set_ptr(p, t); }" % GLOB.match(r[1]).group(2)
        mh = re.fullmatch(r"\*\(\(float \*\)&(v\d+)\+1\)", sl.replace(" ", "")) or re.fullmatch(r"\*\(\(float\*\)&(v\d+)\+1\)", sl.replace(" ", ""))
        if (l[0] == "call" and l[1][0] == "id" and l[1][1] in ("HIDWORD", "SHIDWORD") and l[2][0][0] == "id"
                and self.LT.get(l[2][0][1]) == "i64") or (mh and self.LT.get(mh.group(1)) == "i64"):
            # the high dword of a 64-bit local
            v = l[2][0][1] if l[0] == "call" else mh.group(1)
            rs, rt = self.emit(r)
            if is_ptr(rt):
                hv = "%s.bits()" % rs
            elif mh:
                hv = "(%s).to_bits() as i32" % conv(rs, rt, "f32")
            else:
                hv = conv(rs, rt, "i32")
            return "%s = (%s & 0xFFFF_FFFFi64) | (((%s) as u32 as i64) << 32);" % (v, v, hv)
        if l[0] == "call" and l[1][0] == "id" and l[1][1] in ("HIDWORD", "SHIDWORD") and re.search(r"\+\+|--", show(r)):
            # `HIDWORD(v1) = ++i`: the packing is dropped, the increment is not
            return "{ let _ = %s; }" % self.emit(r)[0]
        m64 = re.fullmatch(r"\*\((float|_DWORD|int) \*\)&(v\d+)", sl)
        if m64 and self.LT.get(m64.group(2)) == "i64":
            # `*(float *)&v64 = x`: store into the low dword
            v = m64.group(2)
            rs, rt = self.emit(r)
            if is_ptr(rt):
                return "%s = (%s & !0xFFFF_FFFFi64) | ((%s).bits() as u32 as i64);" % (v, v, rs)
            if m64.group(1) == "float":
                return "%s = (%s & !0xFFFF_FFFFi64) | ((%s).to_bits() as i64);" % (v, v, self.emit_val(r, "f32"))
            return "%s = (%s & !0xFFFF_FFFFi64) | ((%s) as u32 as i64);" % (v, v, self.emit_val(r, "i32"))
        if "retaddr" in show(r) or (l[0] == "call" and l[1][0] == "id" and l[1][1] in ("HIDWORD", "SHIDWORD")) \
                or l[0] == "un" and re.search(r"&(v\d+)", sl) and self.LT.get(re.search(r"&(v\d+)", sl).group(1)) in ("i64", "f64"):
            return "/* return-value packing */"
        n = self.lodw(l)
        if n is None and self.fidx(l) is not None and self.frame_elem(self.fidx(l)) == "i32":
            n = self.fidx(l)
        if n is not None:
            mhr = re.fullmatch(r"\*\(\(float\*\)&(v\d+)\+1\)", show(r).replace(" ", ""))
            if n in self.ptrslots and mhr and self.LT.get(mhr.group(1)) == "i64":
                # a pointer parked in the high dword of a 64-bit local
                return "{ let t = Ptr::raw((%s >> 32) as i32); cx.set_lpp(%d, t); }" % (mhr.group(1), n)
            if n in self.ptrslots or self.ptrish(r) and is_ptr(self.emit(r)[1]):
                if r[0] == "un" and r[1] == "*" and not is_ptr(self.emit(r)[1]):
                    x = r[2][2] if r[2][0] == "cast" else r[2]
                    return "{ let t = %s; cx.set_lpp(%d, t); }" % (hoist("ptr_at", self.emit_ptr(x)[0]), n)
                s, t = self.emit(r)
                if is_ptr(t):
                    self.ptrslots.add(n)
                    return "{ let t = %s; cx.set_lpp(%d, t); }" % (s, n)
                if n in self.ptrslots and t in ("i32", "ilit"):
                    return "{ let t = %s; cx.set_lpp(%d, Ptr::raw(t)); }" % (s, n)
            s, t = self.emit(r)
            if op != "=":
                return "{ let t = cx.li(%d).%s(%s); cx.set_li(%d, t); }" % (4 * n, IOP[op], conv(s, t, "i32"), 4 * n)
            if t == "f32":
                return "{ let t = %s; cx.set_li(%d, t.to_bits() as i32); }" % (s, 4 * n)
            return "{ let t = %s; cx.set_li(%d, t); }" % (conv(s, t, "i32"), 4 * n)
        n = self.fidx(l)
        if n is not None:
            if n in self.ptrslots and r[0] == "un" and r[1] == "*":
                x = r[2][2] if r[2][0] == "cast" else r[2]
                return "{ let t = %s; cx.set_lpp(%d, t); }" % (hoist("ptr_at", self.emit_ptr(x)[0]), n)
            s, t = self.emit(r)
            if is_ptr(t):
                self.ptrslots.add(n)
                return "{ let t = %s; cx.set_lpp(%d, t); }" % (s, n)
            if self.frame_elem(n) == "f64":
                return "{ let t = %s; cx.set_ld(%d, t); }" % (conv(s, t, "f64"), 8 * n)
            if op != "=":
                return "{ let t = (cx.lf(%d) as f64 %s %s) as f32; cx.set_lf(%d, t); }" % (4 * n, op[0], conv(s, t, "f64"), 4 * n)
            return "{ let t = %s; cx.set_lf(%d, t); }" % (conv(s, t, "f32"), 4 * n)
        if l[0] == "call" and l[1][0] == "id" and l[1][1] in ("LODWORD", "SLODWORD") and l[2][0][0] == "idx" and self.fidx(l[2][0]) is None:
            # LODWORD(p[i]) / LODWORD(frame[dynamic]) as an lvalue: the dword at that element
            x = l[2][0]
            if x[1][0] == "id" and x[1][1] in self.arrays:
                pp = "cx.lptr(%d).add(4 * (%s))" % (self.abase[x[1][1]], self.emit_int(x[2]))
            else:
                b, bt = self.emit_ptr(x[1])
                pp = self.padd(b, bt, self.emit_int(x[2]))
            s2, t2 = self.emit(r)
            if is_ptr(t2):
                return "{ let p = %s; let t = %s; cx.set_ptr(p, t); }" % (pp, s2)
            return "{ let p = %s; let t = %s; cx.set_i(p, t); }" % (pp, conv(s2, t2, "i32") if t2 != "f32" else "%s.to_bits() as i32" % s2)
        if l[0] == "call" and l[1][0] == "id" and l[1][1] in ("LODWORD", "SLODWORD") and l[2][0][0] == "id":
            nm = l[2][0][1]
            s, t = self.emit(r)
            if self.LT.get(nm) == "i64":
                return "%s = (%s & !0xFFFF_FFFFi64) | ((%s) as i32 as u32 as i64);" % (nm, nm, conv(s, t, "i32") if not is_ptr(t) else s + ".bits()")
            if self.LT.get(nm) == "f32" and t != "f32" and not is_ptr(t):
                # LODWORD(float local) = int: the bits, not a conversion (0x0035C980 `LODWORD(v6) = a4 - 1`)
                return "%s = f32::from_bits((%s) as u32);" % (nm, conv(s, t, "i32"))
            return "%s = %s;" % (nm, conv(s, t, self.LT.get(nm, "i32")))
        if l[0] == "call" and l[1][0] == "id" and l[1][1] in ("HIBYTE", "BYTE1", "BYTE2", "HIWORD", "LOBYTE", "LOWORD"):
            # byte writes into a debug flag local
            x = l[2][0]
            if x[0] == "id" and self.LT.get(x[1]) in ("i64", "i32"):
                # part of an integer local (return-value bytes: `LOBYTE(v9) = cond; return v9`)
                sh, w = {"LOBYTE": (0, 0xFF), "BYTE1": (8, 0xFF), "BYTE2": (16, 0xFF), "HIBYTE": (24, 0xFF),
                         "LOWORD": (0, 0xFFFF), "HIWORD": (16, 0xFFFF)}[l[1][1]]
                ty = self.LT[x[1]]
                return "%s = (%s & !(0x%Xi64 as %s)) | ((((%s) as i64) & 0x%X) << %d) as %s;" % (
                    x[1], x[1], w << sh, ty, self.emit_val(r, "i32"), w, sh, ty)
            if x[0] == "id" and is_ptr(self.LT.get(x[1])):
                # byte of a pointer-typed local (bool results parked in v1/v2: 0x001E4AE0, 0x001E4C30)
                sh, w = {"LOBYTE": (0, 0xFF), "BYTE1": (8, 0xFF), "BYTE2": (16, 0xFF), "HIBYTE": (24, 0xFF),
                         "LOWORD": (0, 0xFFFF), "HIWORD": (16, 0xFFFF)}[l[1][1]]
                return "%s = Ptr::raw((%s.bits() & !(0x%Xu32 as i32)) | (((%s) & 0x%X) << %d));" % (
                    x[1], x[1], w << sh, self.emit_val(r, "i32"), w, sh)
            if x[0] == "id" and x[1] in self.LT:
                return "{ let _ = %s; } // %s" % (self.emit(r)[0], show(l))
            n = self.fidx(x) if self.fidx(x) is not None else self.lodw(x)
            if n is not None:
                sh, w = {"LOBYTE": (0, 0xFF), "BYTE1": (8, 0xFF), "BYTE2": (16, 0xFF), "HIBYTE": (24, 0xFF),
                         "HIWORD": (16, 0xFFFF), "LOWORD": (0, 0xFFFF)}[l[1][1]]
                return "{ let t = %s; let o = cx.li(%d); cx.set_li(%d, (o & !(%d << %d)) | ((t & %d) << %d)); }" % (
                    self.emit_int(r), 4 * n, 4 * n, w, sh, w, sh)
        d = self.deref(l)
        if d:
            a, p = d
            s, t = self.emit(r)
            if a == "lvar":
                lt = self.LT[p]
                if lt == "f32bits":
                    return "%s = %s;" % (p, conv(s, t, "f32"))
                return "%s = %s;" % (p, conv(s, t, lt))
            if op == "=":
                if is_ptr(t) and a in ("i", "f"):
                    return "{ let p = %s; let t = %s; cx.set_ptr(p, t); }" % (p, s)
                if a == "i" and t == "f32":
                    return "{ let p = %s; let t = %s; cx.set_i(p, t.to_bits() as i32); }" % (p, s)
                if a == "i" and is_ptr(t):
                    return "{ let p = %s; let t = %s; cx.set_ptr(p, t); }" % (p, s)
                if a == "p":
                    return "{ let p = %s; let t = %s; cx.set_ptr(p, t); }" % (p, s)
                return "{ let p = %s; let t = %s; cx.set_%s(p, t); }" % (p, conv(s, t, ATY[a]), a)
            if a == "f":
                return "{ let p = %s; let t = (cx.f(p) as f64 %s %s) as f32; cx.set_f(p, t); }" % (p, op[0], conv(s, t, "f64"))
            return "{ let p = %s; let t = cx.%s(p).%s(%s); cx.set_%s(p, t); }" % (p, a, IOP[op], conv(s, t, "i32"), a)
        if l[0] == "id" and GLOB.match(l[1]):
            g = GLOB.match(l[1])
            a = {"dword": "i", "flt": "f", "byte": "b", "word": "i16", "dbl": "d"}.get(g.group(1), "i")
            s, t = self.emit(r)
            if op != "=":
                return "{ let p = Ptr::glob(0x%s); let t = cx.%s(p).%s(%s); cx.set_%s(p, t); }" % (g.group(2), a, IOP[op], conv(s, t, "i32"), a)
            return "{ let p = Ptr::glob(0x%s); let t = %s; cx.set_%s(p, t); }" % (g.group(2), conv(s, t, ATY[a]), a)
        if l[0] == "id":
            nm = l[1]
            lt = self.LT.get(nm, "?")
            if lt == "f32bits":
                lt = "f32"
            s, t = self.emit(r)
            if op == "=":
                if is_ptr(lt) and not is_ptr(t):
                    if t in ("i32", "ilit") and s == "0":
                        return "%s = Ptr::NULL;" % nm
                    return "%s = Ptr::raw(%s);" % (nm, conv(s, t, "i32"))
                if lt in ("i32",) and is_ptr(t):
                    self.LT[nm] = "pv"
                    return "%s = %s;" % (nm, s)
                return "%s = %s;" % (nm, conv(s, t, lt) if lt != "?" else s)
            if is_ptr(lt):
                i = conv(s, t, "i32")
                return "%s = %s;" % (nm, self.padd(nm, lt, i if op == "+=" else "-(%s)" % i))
            if lt == "i32":
                return "%s = %s.%s(%s);" % (nm, nm, IOP[op], conv(s, t, "i32"))
            if lt in ("f32", "f64"):
                return "%s = (%s as f64 %s %s) as %s;" % (nm, nm, op[0], conv(s, t, "f64"), lt)
            return "%s %s %s;" % (nm, op, conv(s, t, lt))
        return "/*TODO assign %s*/" % show(("assign", op, l, r))

    def emit_stmt(self, s, ind):
        k = s[0]
        o = []
        if k == "block":
            for x in s[1]:
                o += self.emit_stmt(x, ind)
        elif k == "lblock":
            o.append(ind + "'%s: {" % s[1])
            for x in s[2]:
                o += self.emit_stmt(x, ind + "    ")
            o.append(ind + "}")
        elif k == "bloop":
            # statements after a label that a later goto jumps back to
            self.bscopes = getattr(self, "bscopes", []) + [s[1]]
            o.append(ind + "'%s: loop {" % s[1])
            for x in s[2]:
                o += self.emit_stmt(x, ind + "    ")
            o.append(ind + "    #[allow(unreachable_code)]")
            o.append(ind + "    break;")
            o.append(ind + "}")
            self.bscopes = self.bscopes[:-1]
        elif k == "expr":
            e = s[1]
            if e[0] == "assign":
                o.append(ind + self.emit_assign(e[1], e[2], e[3]))
            elif e[0] == "comma":
                o += self.emit_stmt(("expr", e[1]), ind)
                o += self.emit_stmt(("expr", e[2]), ind)
            elif e[0] == "call" and e[1][0] == "id" and e[1][1] in SKIP:
                pass
            else:
                r, t = self.emit(e)
                o.append(ind + ("let _ = " if not r.startswith("{") and t != "void" else "") + r + ";")
        elif k == "if":
            o.append(ind + "if %s {" % self.emit_val(s[1], "bool"))
            o += self.emit_stmt(s[2], ind + "    ")
            if s[3] is not None:
                if s[3][0] == "if":
                    sub = self.emit_stmt(s[3], ind)
                    o.append(ind + "} else " + sub[0].lstrip())
                    o += sub[1:]
                    return o
                o.append(ind + "} else {")
                o += self.emit_stmt(s[3], ind + "    ")
            o.append(ind + "}")
        elif k == "for":
            if s[1]:
                o += self.emit_stmt(("expr", s[1]), ind)
            lab = self.new_loop()
            o.append(ind + "'%s: loop {" % lab)
            if s[2]:
                o.append(ind + "    if !%s { break '%s; }" % (self.emit_val(s[2], "bool"), lab))
            # `continue` must still run the step: it leaves the body block instead
            self.loops.append((lab, "b" + lab if s[3] else None))
            body = self.emit_stmt(s[4], ind + "        ")
            self.loops.pop()
            if s[3]:
                o.append(ind + "    'b%s: {" % lab)
                o += body
                o.append(ind + "    }")
                o += self.emit_stmt(("expr", s[3]), ind + "    ")
            else:
                o += [x[4:] for x in body]
            o.append(ind + "}")
        elif k == "while":
            lab = self.new_loop()
            o.append(ind + "'%s: while %s {" % (lab, self.emit_val(s[1], "bool")))
            self.loops.append((lab, None))
            o += self.emit_stmt(s[2], ind + "    ")
            self.loops.pop()
            o.append(ind + "}")
        elif k == "do":
            lab = self.new_loop()
            o.append(ind + "'%s: loop {" % lab)
            o.append(ind + "    'b%s: {" % lab)
            self.loops.append((lab, "b" + lab))
            o += self.emit_stmt(s[1], ind + "        ")
            self.loops.pop()
            o.append(ind + "    }")
            o.append(ind + "    if !%s { break '%s; }" % (self.emit_val(s[2], "bool"), lab))
            o.append(ind + "}")
        elif k == "switch":
            o.append(ind + "match %s {" % self.emit_val(s[1], "i32"))
            pend = []
            for vals, st in s[2]:
                pend += vals
                if not st:
                    continue
                body = list(st)
                if body and body[-1][0] == "break":
                    body = body[:-1]
                elif not (body and body[-1][0] in ("return", "goto")):
                    o.append(ind + "    // TODO fallthrough")
                pat = " | ".join("_" if x == "default" else self.emit(x)[0] for x in pend)
                pend = []
                o.append(ind + "    %s => {" % pat)
                for x in body:
                    o += self.emit_stmt(x, ind + "        ")
                o.append(ind + "    }")
            if pend:
                o.append(ind + "    %s => {}" % " | ".join("_" if x == "default" else self.emit(x)[0] for x in pend))
            if not any("default" in v for v, _ in s[2]):
                o.append(ind + "    _ => {}")
            o.append(ind + "}")
        elif k == "return":
            if s[1] is None:
                o.append(ind + "return R0;")
            else:
                rs, rt = self.emit(s[1])
                if rt == "void" and self.rk() != "void":
                    o.append(ind + rs + ";")
                    o.append(ind + "return R0;")
                else:
                    o.append(ind + "return %s;" % conv(rs, rt, self.rk()))
        elif k == "break":
            o.append(ind + ("break '%s;" % self.loops[-1][0] if self.loops else "break;"))
        elif k == "continue":
            lab, body = self.loops[-1]
            o.append(ind + ("break '%s;" % body if body else "continue '%s;" % lab))
        elif k == "goto":
            if "b_" + s[1] in getattr(self, "bscopes", []):
                o.append(ind + "continue 'b_%s;" % s[1])
            else:
                o.append(ind + "break 'l_%s;" % s[1])
        elif k == "label":
            o.append(ind + "// %s:" % s[1])
        return o

    # ---- CFG lowering (functions whose gotos jump into nested blocks)
    def emit_cfg(self, top):
        self.cfg_blocks = [[]]
        self.cfg_cur = 0
        self.cfg_labels = {}
        self.cfg_gen(top, None, None)
        self.cfg_term("break 'cfg;")
        o = ["    let mut bb: u32 = 0;", "    #[allow(unreachable_code)]", "    'cfg: loop {", "        match bb {"]
        for i, b in enumerate(self.cfg_blocks):
            o.append("            %d => {" % i)
            o += ["                " + x.lstrip() if not x.startswith(" " * 16) else x for x in b]
            if not b or not self.cfg_is_term(b[-1]):
                o.append("                break 'cfg;")
            o.append("            }")
        o.append("            _ => unreachable!(),")
        o.append("        }")
        o.append("    }")
        return o

    def cfg_is_term(self, line):
        l = line.strip()
        return l.startswith("bb = ") and l.endswith("continue 'cfg; }") or l in ("break 'cfg;",) or l.startswith("return ") or l.startswith("match ") and l.endswith("}")

    def cfg_new(self):
        self.cfg_blocks.append([])
        return len(self.cfg_blocks) - 1

    def cfg_jump(self, b):
        return "{ bb = %d; continue 'cfg; }" % b

    def cfg_term(self, line):
        """end the current block with `line`; following code goes to a fresh (unreachable) block"""
        self.cfg_blocks[self.cfg_cur].append(line)
        self.cfg_cur = self.cfg_new()

    def cfg_goto(self, b):
        self.cfg_blocks[self.cfg_cur].append("bb = %d; continue 'cfg; }" % b if False else "{ bb = %d; continue 'cfg; }" % b)
        self.cfg_cur = self.cfg_new()

    def cfg_label(self, name):
        if name not in self.cfg_labels:
            self.cfg_labels[name] = self.cfg_new()
        return self.cfg_labels[name]

    def cfg_enter(self, b):
        """fall through into block b"""
        self.cfg_blocks[self.cfg_cur].append(self.cfg_jump(b))
        self.cfg_cur = b

    def cfg_gen(self, s, brk, cont):
        if s is None:
            return
        k = s[0]
        cur = lambda: self.cfg_blocks[self.cfg_cur]
        if k == "block":
            for x in s[1]:
                self.cfg_gen(x, brk, cont)
        elif k == "if":
            t, e, j = self.cfg_new(), self.cfg_new(), self.cfg_new()
            cur().append("if %s %s else %s" % (self.emit_val(s[1], "bool"), self.cfg_jump(t), self.cfg_jump(e)))
            self.cfg_cur = t
            self.cfg_gen(s[2], brk, cont)
            self.cfg_enter(j) if False else cur().append(self.cfg_jump(j))
            self.cfg_cur = e
            self.cfg_gen(s[3], brk, cont)
            cur().append(self.cfg_jump(j))
            self.cfg_cur = j
        elif k == "while":
            h, b, x = self.cfg_new(), self.cfg_new(), self.cfg_new()
            self.cfg_enter(h)
            cur().append("if %s %s else %s" % (self.emit_val(s[1], "bool"), self.cfg_jump(b), self.cfg_jump(x)))
            self.cfg_cur = b
            self.cfg_gen(s[2], x, h)
            cur().append(self.cfg_jump(h))
            self.cfg_cur = x
        elif k == "do":
            b, c, x = self.cfg_new(), self.cfg_new(), self.cfg_new()
            self.cfg_enter(b)
            self.cfg_gen(s[1], x, c)
            self.cfg_enter(c)
            cur().append("if %s %s else %s" % (self.emit_val(s[2], "bool"), self.cfg_jump(b), self.cfg_jump(x)))
            self.cfg_cur = x
        elif k == "for":
            if s[1]:
                cur().extend(self.emit_stmt(("expr", s[1]), ""))
            h, b, st, x = self.cfg_new(), self.cfg_new(), self.cfg_new(), self.cfg_new()
            self.cfg_enter(h)
            if s[2]:
                cur().append("if %s %s else %s" % (self.emit_val(s[2], "bool"), self.cfg_jump(b), self.cfg_jump(x)))
            else:
                cur().append(self.cfg_jump(b))
            self.cfg_cur = b
            self.cfg_gen(s[4], x, st)
            self.cfg_enter(st)
            if s[3]:
                cur().extend(self.emit_stmt(("expr", s[3]), ""))
            cur().append(self.cfg_jump(h))
            self.cfg_cur = x
        elif k == "switch":
            x = self.cfg_new()
            arms = []
            dflt = x
            case_blocks = []
            pend = []
            for vals, st in s[2]:
                b = self.cfg_new()
                case_blocks.append((b, st))
                for v in vals:
                    if v == "default":
                        dflt = b
                    else:
                        arms.append((self.emit(v)[0], b))
            m = "match %s { %s _ => %s }" % (self.emit_val(s[1], "i32"),
                " ".join("%s => %s" % (v, self.cfg_jump(b)) for v, b in arms), self.cfg_jump(dflt))
            cur().append(m)
            for idx, (b, st) in enumerate(case_blocks):
                self.cfg_cur = b
                for y in st:
                    self.cfg_gen(y, x, cont)
                nxt = case_blocks[idx + 1][0] if idx + 1 < len(case_blocks) else x
                cur().append(self.cfg_jump(nxt))
            self.cfg_cur = x
        elif k == "break":
            self.cfg_goto(brk)
        elif k == "continue":
            self.cfg_goto(cont)
        elif k == "goto":
            self.cfg_goto(self.cfg_label(s[1]))
        elif k == "label":
            self.cfg_enter(self.cfg_label(s[1]))
        elif k == "return":
            for l in self.emit_stmt(s, ""):
                cur().append(l)
            self.cfg_cur = self.cfg_new()
        elif k in ("lblock", "bloop"):
            for x in s[2]:
                self.cfg_gen(x, brk, cont)
        else:
            cur().extend(self.emit_stmt(s, ""))

    def new_loop(self):
        self.nloop += 1
        return "lp%d" % self.nloop

    def rk(self):
        r = self.ret
        return {"void": "void", "f32": "f32", "f64": "f64", "i64": "i64"}.get(r, "pv" if is_ptr(r) else "i32")

    def translate(self):
        body = self.body
        # virtual-call casts: keep the return type as a marker type `vfn_<ret>`
        body = re.sub(r"\((void|int|char|float|double|_DWORD \*|__int64|_DWORD) \(__thiscall \*\*\*?\)\([^()]*\)\)",
                      lambda m: "(vfn_%s **)" % {"_DWORD *": "ptr", "__int64": "int64"}.get(m.group(1), m.group(1).replace(" ", "")), body)
        body = re.sub(r"\((?:void|int|char|float|double|_DWORD \*|__int64|_DWORD) \(__thiscall \*\)\([^()]*\)\)", "(void *)", body)
        lines = []
        for l in body.split("\n"):
            if re.match(r"^\s*([A-Za-z_][\w ]*?)\s*\**\s*\w+(\[\d+\])?;\s*//", l) or re.match(r"^\s*_UNKNOWN \*retaddr;", l):
                continue
            if self.frame and re.search(r"memset\(%s, (0xCCu|204)," % self.frame, l):
                continue
            lines.append(l)
        body = "\n".join(lines)
        p = P(tokenize(body))
        try:
            top = p_stmt(p)
        except Exception as ex:
            raise SystemExit("%s: %s near %s" % (self.name, ex, " ".join(str(x[1]) for x in p.t[max(0, p.i - 30):p.i + 5])))
        if needs_cfg(top):
            # gotos into nested blocks: lower the whole body to a block-dispatch loop
            out = self.emit_cfg(top)
            out = self.emit_cfg(top)   # second pass: pointer slots found late
        else:
            top = fix_tree(top)
            out = self.emit_stmt(top, "    ")
            out = self.emit_stmt(top, "    ")   # second pass: pointer slots found late
        txt = "\n".join(out)
        rk = self.rk()
        rty = {"void": "()", "f32": "f32", "f64": "f64", "i64": "i64", "i32": "i32", "pv": "Ptr"}[rk]
        r0 = {"void": "()", "f32": "0.0", "f64": "0.0", "i64": "0", "i32": "0", "pv": "Ptr::NULL"}[rk]
        txt = txt.replace("return R0;", "return %s;" % r0)
        locs = []
        for nm, t in sorted(self.LT.items(), key=lambda x: (len(x[0]), x[0])):
            if any(nm == pn for _, pn in self.params):
                continue
            if re.search(r"\b%s\b" % nm, txt):
                init = {"f32": "0.0f32", "f32bits": "0.0f32", "f64": "0.0f64", "i32": "0i32", "i64": "0i64"}.get(t, "Ptr::NULL")
                locs.append("    let mut %s = %s;" % (nm, init))
        ps = []
        for k, n in self.params:
            ps.append("%s%s: %s" % ("mut " if re.search(r"\b%s\s*(=[^=]|\+=|-=|\*=|/=|\+\+|--)" % n, self.body) or re.search(r"(\+\+|--)%s\b" % n, self.body) else "", n,
                                    {"f32": "f32", "f64": "f64", "i32": "i32", "i64": "i64"}.get(k, "Ptr")))
        for k, n in self.params:
            if self.LT.get(n) == "f32bits":
                txt = "    let mut %s = f32::from_bits(%s as u32);\n" % (n, n) + txt
        fname = "f_" + self.addr.lower()
        hdr = "/// %s 0x%08X\n" % (self.name, int(self.addr, 16))
        if self.frame_size:
            s = hdr + "pub fn %s(cx: &mut Cx%s) -> %s {\n    cx.push_frame(%d);\n    let r = %s_body(cx%s);\n    cx.pop_frame();\n    r\n}\n\n" % (
                fname, "".join(", " + x for x in ps), rty, self.frame_size, fname, "".join(", " + x.split(":")[0].replace("mut ", "") for x in ps))
            s += "#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]\nfn %s_body(cx: &mut Cx%s) -> %s {\n" % (fname, "".join(", " + x for x in ps), rty)
        else:
            s = hdr + "#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]\npub fn %s(cx: &mut Cx%s) -> %s {\n" % (fname, "".join(", " + x for x in ps), rty)
        s += "\n".join(locs) + ("\n" if locs else "") + txt + "\n"
        if rk != "void" and not re.search(r"^    return [^\n]*;\s*$", txt.rstrip().split("\n")[-1]):
            s += "    #[allow(unreachable_code)]\n    %s\n" % r0
        s += "}\n"
        return s

PATCHES = {}
try:
    _pt = open(ROOT + r"\tools\phys_c2rs4_patches.txt", encoding="utf-8").read()
    for m in re.finditer(r"(?ms)^=== (\w+)\n<<<\n(.*?)\n---\n(.*?)\n>>>$", _pt):
        PATCHES.setdefault(m.group(1).upper(), []).append((m.group(2), m.group(3)))
except FileNotFoundError:
    pass

SKIP = {"InputDevice__3DDC60", "InputDevice__3DDC30", "__debugbreak", "sub_3E1FB0"}

STEM_MOD = {"Extra": "vehicle/vehicle_entity/tr_ext.rs", "MaterialProperties": "vehicle/vehicle_entity/tr_ext.rs", "Redump": "vehicle/vehicle_entity/tr_ext.rs",
            "VehicleDatabaseVehicleBinLoading": "vehicle/vehicle_entity/tr_ext.rs", "CarDamage": "vehicle/vehicle_entity/tr_ext.rs",
            "VehicleEntity": "vehicle/vehicle_entity/tr.rs", "FastCarLib": "fastcar/tr.rs",
            "PhysicsManager": "physics_manager/tr.rs", "PhysicsCarInput": "vehicle/physics_car_input/tr.rs",
            "VsConfigFileRead": "vehicle/vs_config_file_read/tr.rs", "VehicleManager": "vehicle/vehicle_manager/tr.rs",
            "PhysicsObjectManager": "physics_objects/physics_object_manager/tr.rs",
            "PhysicsObjectParticleTracker": "physics_objects/physics_object_particle_tracker/tr.rs",
            "PhysicsObjects": "physics_objects/physics_objects/tr.rs"}

IOP = {"+=": "wrapping_add", "-=": "wrapping_sub", "*=": "wrapping_mul", "/=": "wrapping_div", "|=": "bitor_", "&=": "bitand_",
       "^=": "bitxor_", "<<=": "shl_", ">>=": "shr_"}
FIELDS = {}
for _l in open(ROOT + r"\research\phys_ve_fields.txt"):
    _n, _o = _l.split()
    FIELDS[_n] = int(_o)

def hoist(acc, p):
    """cx.<acc>(p); a pointer computed by a call that needs `&mut cx` is bound first"""
    if re.search(r"\w\(cx[,)]", p):
        return "{ let p = %s; cx.%s(p) }" % (p, acc)
    return "cx.%s(%s)" % (acc, p)

def conv(s, t, want):
    if t == want or want is None or want == "?":
        return s
    if t == "str":
        return "Ptr::NULL /*str*/" if is_ptr(want) else "0 /*str*/" if want in ("i32", "i64") else s
    if t == "void":
        return "{ %s; %s }" % (s, {"f32": "0.0f32", "f64": "0.0f64", "i64": "0i64", "bool": "false"}.get(want, "Ptr::NULL" if is_ptr(want) else "0"))
    if t == "f32bits":
        t = "f32"
    if t == "bool" and want in ("i32", "i64"):
        return "(%s as %s)" % (s, want)
    if t == "lit":
        if want in ("f32", "f64"):
            return s + want
        if want == "i32":
            return "(%s as i32)" % (s + "f64")
        return s
    if t == "ilit":
        if want in ("f32", "f64"):
            return s + ".0" + want if "." not in s else s + want
        if want == "bool":
            return "(%s != 0)" % s
        if want == "i64":
            return s + "i64"
        return s
    if want in ("f64", "f32"):
        if t in ("f32", "f64", "i32", "i64"):
            return "(%s as %s)" % (s, want)
        if t == "bool":
            return "(%s as i32 as %s)" % (s, want)
    if want == "i32":
        if t in ("f32", "f64"):
            return "(%s as i32)" % s
        if t in ("bool", "i64"):
            return "(%s as i32)" % s
        if is_ptr(t):
            return "%s.bits()" % s
        return s
    if want == "i64":
        if t in ("i32", "bool", "f32", "f64"):
            return "(%s as i64)" % s
        return s
    if want == "bool":
        if t in ("f32", "f64"):
            return "(%s != 0.0)" % s
        if is_ptr(t):
            return "!%s.is_null()" % s
        return "(%s != 0)" % s
    if is_ptr(want) and not is_ptr(t):
        if s == "0":
            return "Ptr::NULL"
        return "Ptr::raw(%s)" % conv(s, t, "i32")
    return s

def numlit(v):
    v = v.rstrip("uLl")
    if v.startswith("0x"):
        return str(int(v, 16) - (1 << 32) if int(v, 16) >= 1 << 31 else int(v, 16)), "ilit"
    if "." in v or "e" in v.lower():
        f = v.rstrip("fF")
        if f.endswith("."):
            f += "0"
        if f.startswith("."):
            f = "0" + f
        if "e" in f.lower() and "." not in f.split("e")[0].split("E")[0]:
            f = re.sub(r"[eE]", ".0e", f, 1)
        return f, "lit"
    iv = int(v)
    if iv >= 1 << 31:
        iv -= 1 << 32
    return str(iv), "ilit"


# ---- gotos into a nested block: Hex-Rays merges tails, so `goto L` jumps into the middle of another
# block. When the code after `L` ends in return/goto (or the function), the goto is replaced by a copy
# of that tail; gotos inside the label's own statement list stay (labeled blocks / loops handle them).
TERMINAL = ("return", "goto")

def _children_lists(s):
    """(list, rebuild) pairs of the statement lists directly inside s"""
    k = s[0]
    if k == "block":
        return [s[1]]
    if k in ("lblock", "bloop"):
        return [s[2]]
    if k == "switch":
        return [st for _, st in s[2]]
    return []

def _stmts(s):
    k = s[0]
    if k == "if":
        return [x for x in (s[2], s[3]) if x is not None]
    if k == "for":
        return [s[4]]
    if k == "while":
        return [s[1 + 1]]
    if k == "do":
        return [s[1]]
    return []

def _scan(s, path, labels, gotos, top):
    if s is None or not isinstance(s, tuple):
        return
    for lst in _children_lists(s):
        for i, x in enumerate(lst):
            if x and x[0] == "label":
                labels[x[1]] = (lst, i, lst is top)
            _scan(x, path + [id(lst)], labels, gotos, top)
    for x in _stmts(s):
        if x and x[0] == "goto":
            gotos.append((x[1], path))
        _scan(x, path, labels, gotos, top)
    if s[0] == "goto":
        gotos.append((s[1], path))

def needs_cfg(top):
    labels, gotos = {}, []
    _scan(top, [], labels, gotos, top[1] if top[0] == "block" else None)
    for lab, path in gotos:
        if lab not in labels or id(labels[lab][0]) not in path:
            return True
    return False

def dup_tails(top):
    import copy
    labels, gotos = {}, []
    top_list = top[1] if top[0] == "block" else None
    _scan(top, [], labels, gotos, top_list)
    tails = {}
    for lab, (lst, i, is_top) in labels.items():
        bad = [g for g in gotos if g[0] == lab and id(lst) not in g[1]]
        if not bad:
            continue
        seg = []
        ok = False
        for x in lst[i + 1:]:
            if x[0] == "label":
                seg.append(x)
                continue
            seg.append(x)
            if x[0] in TERMINAL:
                ok = True
                break
        if not ok and is_top:
            ok = True
        if ok and not any(x[0] == "label" for x in seg):
            tails[(lab, id(lst))] = seg
    if not tails:
        return top
    def tr(s, path):
        if s is None or not isinstance(s, tuple):
            return s
        k = s[0]
        if k == "goto":
            for (lab, lid), seg in tails.items():
                if lab == s[1] and lid not in path:
                    return ("block", copy.deepcopy(seg))
            return s
        if k == "block":
            return ("block", [tr(x, path + [id(s[1])]) for x in s[1]])
        if k in ("lblock", "bloop"):
            return (k, s[1], [tr(x, path + [id(s[2])]) for x in s[2]])
        if k == "switch":
            return ("switch", s[1], [(v, [tr(x, path + [id(st)]) for x in st]) for v, st in s[2]])
        if k == "if":
            return ("if", s[1], tr(s[2], path), tr(s[3], path))
        if k == "for":
            return ("for", s[1], s[2], s[3], tr(s[4], path))
        if k == "while":
            return ("while", s[1], tr(s[2], path))
        if k == "do":
            return ("do", tr(s[1], path), s[2])
        return s
    return tr(top, [])

# ---- goto -> labeled blocks (forward gotos only)
def contains_goto(s, lab):
    if isinstance(s, tuple):
        if s and s[0] == "goto":
            return s[1] == lab
        return any(contains_goto(x, lab) for x in s[1:])
    if isinstance(s, list):
        return any(contains_goto(x, lab) for x in s)
    return False

def fix_gotos(lst):
    i = 0
    while i < len(lst):
        s = lst[i]
        if s[0] == "label":
            lab = s[1]
            ks = [k for k in range(i + 1, len(lst)) if contains_goto(lst[k], lab)]
            if ks:
                lst[i + 1:ks[-1] + 1] = [("bloop", "b_" + lab, lst[i + 1:ks[-1] + 1])]
            j = 0
            while j < i and not contains_goto(lst[j], lab):
                j += 1
            if j < i:
                blk = ("lblock", "l_" + lab, lst[j:i])
                lst[j:i + 1] = [blk]
                i = j
        i += 1
    for idx, s in enumerate(lst):
        lst[idx] = fix_tree(s)
    return lst

def fix_tree(s):
    if s is None:
        return s
    k = s[0]
    if k == "block":
        return ("block", fix_gotos(list(s[1])))
    if k == "lblock":
        return ("lblock", s[1], fix_gotos(list(s[2])))
    if k == "bloop":
        return ("bloop", s[1], fix_gotos(list(s[2])))
    if k == "if":
        return ("if", s[1], fix_tree(s[2]), fix_tree(s[3]))
    if k == "for":
        return ("for", s[1], s[2], s[3], fix_tree(s[4]))
    if k == "while":
        return ("while", s[1], fix_tree(s[2]))
    if k == "do":
        return ("do", fix_tree(s[1]), s[2])
    if k == "switch":
        return ("switch", s[1], [(v, fix_gotos(list(st))) for v, st in s[2]])
    return s

if __name__ == "__main__":
    out = sys.argv[1]
    closure = "--closure" in sys.argv
    stop = set()
    args = [a for a in sys.argv[2:] if a != "--closure"]
    if "--stop" in args:
        i = args.index("--stop")
        stop = set(x.upper().lstrip("0") for x in args[i + 1:])
        args = args[:i]
    addrs = [a.upper().lstrip("0") for a in args]
    res = []
    calls = set()
    i = 0
    while i < len(addrs):
        a = addrs[i]
        t = T(a)
        res.append(t.translate())
        calls |= t.calls
        if closure:
            for c in sorted(t.calls):
                if c in FUNCS and c not in addrs and c not in stop:
                    addrs.append(c)
        i += 1
    print("translated:", " ".join(addrs))
    if out.endswith(".rs"):
        open(out, "w", encoding="utf-8").write("\n".join(res))
    else:
        files = {}
        for a, r in zip(addrs, res):
            files.setdefault(STEM_MOD.get(STEM[a], "UNMAPPED_" + STEM[a]), []).append(r)
        for rel, rs in files.items():
            hdr = ("//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the\n"
                   "//! translator or in the hand-written overrides).\n#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]\n\n"
                   "use crate::tr_prelude::*;\n\n")
            path = out + "\\" + rel.replace("/", "\\")
            open(path, "w", encoding="utf-8").write(hdr + "\n".join(rs))
            print("wrote", rel, len(rs))
    if not out.endswith(".rs"):
        # virtual-call dispatch: every translated method by address
        vd = ["//! Virtual-call dispatch over the translated methods (generated by tools/phys_c2rs4.py).",
              "#![allow(unused_variables, clippy::all)]", "", "use crate::tr_prelude::*;", "",
              "/// call the translated function at `addr` with `this` and the virtual call's arguments",
              "pub fn vdispatch(cx: &mut Cx, addr: u32, this: Ptr, args: &[Varg]) -> Varg {",
              "    match addr {"]
        for a in addrs:
            if a not in SIGS or (a.upper() in EXTRA):
                continue
            rt, ps = fix_sig(a)
            if not ps or ps[0][1] != "this":
                continue
            conv_ = []
            for i, (k, n) in enumerate(ps[1:]):
                kk = {"f32": "f", "f64": "d", "i32": "i", "i64": "q"}.get(k, "p")
                conv_.append("args.get(%d).copied().unwrap_or(Varg::I(0)).%s()" % (i, kk))
            c = "f_%s(cx, this%s)" % (a.lower(), "".join(", " + x for x in conv_))
            w = {"f32": "Varg::F(%s)", "f64": "Varg::F(%s as f32)", "i32": "Varg::I(%s)", "i64": "Varg::I(%s as i32)", "void": "{ %s; Varg::I(0) }"}.get(rt, "Varg::P(%s)")
            vd.append("        0x%s => %s," % (a, w % c))
        vd += ["        _ => panic!(\"virtual call into {addr:#X}: not translated\"),", "    }", "}", ""]
        open(out + "\\physics_manager\\vdispatch.rs", "w", encoding="utf-8").write("\n".join(vd))
    missing = sorted(c for c in calls if c not in addrs)
    print("callees not in this batch:", " ".join(missing))
    todo = sum(r.count("TODO") + r.count("?ptr") for r in res)
    print("TODO/?ptr:", todo)
