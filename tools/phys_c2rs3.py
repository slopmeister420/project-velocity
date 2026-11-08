# physics agent: Hex-Rays -> Rust translator for VehicleEntity functions (memory-space Ptr model).
# usage: python tools/phys_c2rs3.py <in.c> <rust_fn_name> [> out.rs]
# Every pointer is a `Ptr` (space + byte offset); memory goes through `cx` (see vehicle_entity/cx.rs).
import re, sys
sys.path.insert(0, r"D:\projects\project poland rewrite\tools")
from phys_cparse import tokenize, P, p_stmt

src = open(sys.argv[1], encoding="utf-8").read()
src = re.sub(r"(\w+)::(\w+)", r"\1__\2", src)
FNAME = sys.argv[2]

# ---- GC field types
cc = open(r"D:\projects\project poland rewrite\crates\velocity_physics\src\c_config.rs", encoding="utf-8").read()
g = cc[cc.index("pub struct CGlobalConfig"):cc.index("pub enum LoadResult")]
GCT = {}
for m in re.finditer(r"pub (\w+): (.+?), // \+0x([0-9A-F]+)", g):
    n, ty, o = m.groups()
    ty = ty.strip()
    GCT[n] = ("f32" if ty == "f32" else "i32" if ty == "i32" else "expr" if ty == "Expression" else
              "af32" if ty.startswith("[f32") else "ai32" if ty.startswith("[i32") else "?", int(o, 16))

# ---- helper calls: name -> (rust, ret type, arg kinds)  (kinds: p ptr, f f32, i i32, F f64)
CALLS = {
    "VsSin": ("vsmath::sin", "f64", "f"), "VsCos": ("vsmath::cos", "f64", "f"), "VsSqrt": ("vsmath::sqrt", "f64", "f"),
    "VsFabs": ("vsmath::fabs", "f64", "f"), "VsTan": ("vsmath::tan", "f64", "f"), "VsAtan": ("vsmath::atan", "f64", "f"),
    "VsAtan2": ("vsmath::atan2", "f64", "ff"), "VsACos": ("vsmath::acos", "f64", "f"), "VsAsin": ("vsmath::asin", "f64", "f"),
    "VsExp": ("vsmath::exp", "f64", "f"), "VsPow": ("vsmath::pow", "f64", "ff"), "VsAbs": ("i32::abs", "i32", "i"),
    "VsFge": ("vs_fge", "bool", "fff"),
    "VsConfigFileRead__1B6E80": ("hist_entry", "ptr1", "pi"),
    "VsConfigFileRead__1B6DB0": ("hist_push", "void", "p"),
    "VsAIRaceVehicle__248C0": ("car_rot_row", "ptr1", "pi"),
    "VsAIRaceVehicle__24200": ("car_velocity", "ptr1", "p"),
    "PhysicsManager__1B4710": ("car_omega", "ptr1", "p"),
    "VsAIRaceVehicle__24850": ("car_i2876", "i32", "p"),
    "Speedo__F79A0": ("ve_f148", "f64", "p"),
}
KW = {"if", "while", "for", "switch", "return", "LODWORD", "SLODWORD", "HIBYTE", "LOBYTE", "BYTE1", "memset", "sizeof"}
for n in re.findall(r"\b([A-Za-z_]\w*)\(", src[src.index("{"):]):
    if n not in CALLS and n not in KW:
        CALLS[n] = ("h_" + (n.split("__")[-1].lower() if re.search(r"__[0-9A-F]{5,6}$", n) else n.lower()), "?", None)
# explicit return types / arg kinds of translated helpers (filled as they are ported)
RET = {}
try:
    for line in open(r"D:\projects\project poland rewrite\tools\phys_c2rs3_sigs.txt"):
        p = line.split()
        if len(p) >= 2 and not line.startswith("#"):
            RET[p[0]] = (p[1], p[2] if len(p) > 2 else None)
except FileNotFoundError:
    pass
for k, (rt, ak) in RET.items():
    if k in CALLS:
        CALLS[k] = (CALLS[k][0], rt, ak if ak != "-" else "")

# ---- local declarations
LT = {}
for m in re.finditer(r"^\s*(double|float|int|char|bool|_BYTE|__int64|unsigned int|_DWORD \*|float \*|void \*|int \*|_BYTE \*|char \*|unsigned __int8)\s*(\**)\s*(\w+)(\[\d+\])?;", src, re.M):
    ty, star, n, arr = m.groups()
    if arr:
        continue
    if "*" in ty or star:
        LT[n] = "ptr4" if ty.startswith(("float", "_DWORD", "int")) else "ptr1"
    elif ty == "double":
        LT[n] = "f64"
    elif ty == "float":
        LT[n] = "f32"
    elif ty == "__int64":
        LT[n] = "i64"
    else:
        LT[n] = "i32"
for m in re.finditer(r"^\s*(float|int) (\w+); // e[a-z]x", src, re.M):
    pass

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
    return str(e)

def v132(e):
    """v132[N] -> N"""
    if e[0] == "idx" and e[1] == ("id", "v132") and e[2][0] == "num":
        return int(e[2][1])
    return None

def lodw(e):
    """LODWORD(v132[N]) / SLODWORD -> N"""
    if e[0] == "call" and e[1][0] == "id" and e[1][1] in ("LODWORD", "SLODWORD") and len(e[2]) == 1:
        return v132(e[2][0])
    return None

def tsize(ty):
    t = ty.replace(" ", "")
    if t in ("float*", "_DWORD*", "int*", "unsignedint*", "float**", "_DWORD**", "int**"):
        return 4
    if t in ("__int16*", "_WORD*"):
        return 2
    return 1

def acc(ty):
    t = ty.replace(" ", "")
    return {"float*": "f", "_DWORD*": "i", "int*": "i", "unsignedint*": "i", "_BYTE*": "b", "char*": "b",
            "unsigned__int8*": "b", "bool*": "b", "__int16*": "i16", "_WORD*": "i16", "float**": "p", "_DWORD**": "p",
            "int**": "p", "void**": "p"}.get(t, "i")

ATY = {"f": "f32", "i": "i32", "b": "i32", "i16": "i32", "p": "ptr4"}

def is_ptr(t):
    return t in ("ptr1", "ptr4")

def ptrish(e):
    k = e[0]
    n = lodw(e)
    if n is not None:
        return n in PTRSLOTS
    if k == "id":
        return LT.get(e[1]) in ("ptr1", "ptr4") or e[1] == "this"
    if k == "un" and e[1] == "&":
        return True
    if k == "un" and e[1] == "*":
        return True   # a value loaded in pointer context is a pointer
    if k == "cast":
        return e[1].replace(" ", "").endswith("*") or ptrish(e[2])
    if k == "call" and e[1][0] == "id" and e[1][1] in CALLS:
        return CALLS[e[1][1]][1] in ("ptr1", "ptr4")
    if k == "bin" and e[1] in "+-":
        return ptrish(e[2]) or (e[1] == "+" and ptrish(e[3]))
    if k == "field":
        return show(e[1]) == "this"
    return False

def emit_ptr(e):
    """emit e as a pointer expression -> (rust, ptrtype)"""
    k = e[0]
    if k == "un" and e[1] == "*" and not (e[2][0] == "cast" and acc(e[2][1]) in ("f", "b", "i16")):
        inner = e[2]
        p = emit_ptr(inner[2] if inner[0] == "cast" else inner)[0]
        return "cx.ptr_at(%s)" % p, "ptr1"
    if k == "cast" and (e[1].replace(" ", "").endswith("*") or e[1] in ("int", "_DWORD")) and ptrish(e[2]):
        s, t = emit_ptr(e[2])
        return s, ("ptr4" if tsize(e[1]) == 4 and e[1].replace(" ", "").endswith("*") else "ptr1") if e[1].replace(" ", "").endswith("*") else t
    if k == "bin" and e[1] in "+-":
        if ptrish(e[2]):
            a, ta = emit_ptr(e[2])
            i = emit_int(e[3])
            if e[1] == "-":
                i = "-(%s)" % i
            return ("%s.add(%s)" % (a, i) if ta == "ptr1" else "%s.add(4 * (%s))" % (a, i)), ta
        if e[1] == "+" and ptrish(e[3]):
            a, ta = emit_ptr(e[3])
            i = emit_int(e[2])
            return ("%s.add(%s)" % (a, i) if ta == "ptr1" else "%s.add(4 * (%s))" % (a, i)), ta
    s, t = emit(e)
    if is_ptr(t):
        return s, t
    if t == "i32" and s == "0":
        return "Ptr::NULL", "ptr1"
    return "/*?ptr*/(%s)" % s, "ptr1"

GLOB = re.compile(r"^(dword|flt|byte|word|unk|off|qword)_([0-9A-F]+)$")

def deref(e):
    """*(T *)(addr) -> (accessor, ptr rust)"""
    if e[0] == "un" and e[1] == "*":
        inner = e[2]
        if inner[0] == "cast":
            a = acc(inner[1])
            x = inner[2]
            if x[0] == "un" and x[1] == "&" and x[2][0] == "id" and x[2][1] in LT:
                return "lvar", x[2][1]
            p, pt = emit_ptr(inner[2])
            return a, p
        p, pt = emit_ptr(inner)
        return ("f" if pt == "ptr4" and inner[0] == "id" and LT.get(inner[1]) == "ptr4" else "i"), p
    if e[0] == "idx" and v132(e) is None:
        b, bt = emit(e[1])
        if is_ptr(bt):
            i = emit_int(e[2])
            return "f", "%s.add(4 * (%s))" % (b, i) if not re.match(r"^\d+$", i) else "%s.add(%d)" % (b, 4 * int(i))
    return None

def emit_int(e):
    s, t = emit(e)
    return conv(s, t, "i32")

def conv(s, t, want):
    if t == want or want is None:
        return s
    if t == "lit":
        if want in ("f32", "f64"):
            return s + want
        if want == "i32":
            return s
    if t == "ilit":
        if want in ("f32", "f64"):
            return s + ".0" + want
        if want == "bool":
            return "(%s != 0)" % s
        return s
    if want in ("f64", "f32"):
        if t in ("f32", "f64", "i32", "i64"):
            return "(%s as %s)" % (s, want)
        if t == "bool":
            return "(%s as i32 as %s)" % (s, want)
    if want == "i32":
        if t in ("f32", "f64"):
            return "(%s as i32)" % s
        if t == "bool":
            return "(%s as i32)" % s
        if t == "i64":
            return "(%s as i32)" % s
        if is_ptr(t):
            return "%s.bits()" % s
        return s
    if want == "bool":
        if t in ("f32", "f64"):
            return "(%s != 0.0)" % s
        if is_ptr(t):
            return "!%s.is_null()" % s
        return "(%s != 0)" % s
    return s

def numlit(v):
    v = v.rstrip("u")
    if v.startswith("0x"):
        return str(int(v, 16)), "ilit"
    if "." in v or ("e" in v.lower()):
        f = v.rstrip("fF")
        if "." not in f and "e" not in f.lower():
            f += ".0"
        if f.endswith("."):
            f += "0"
        if f.startswith("."):
            f = "0" + f
        if "e" in f.lower() and "." not in f:
            f = f.lower().replace("e", ".0e")
        return f, "lit"
    return v, "ilit"

def emit(e):
    """-> (rust, type) type in f32 f64 i32 i64 bool ptr1 ptr4 lit ilit expr void ?"""
    k = e[0]
    n = lodw(e)
    if n is not None:
        return "cx.lp[%d]" % n if n in PTRSLOTS else "cx.l.i(%d)" % (4 * n), ("ptr1" if n in PTRSLOTS else "i32")
    n = v132(e)
    if n is not None:
        return "cx.l.f(%d)" % (4 * n), "f32"
    if k == "un" and e[1] == "&":
        n = v132(e[2])
        if n is not None:
            return "Ptr::l(%d)" % (4 * n), "ptr4"
        if e[2][0] == "id" and GLOB.match(e[2][1]):
            return "Ptr::glob(0x%s)" % GLOB.match(e[2][1]).group(2), "ptr1"
        if e[2][0] == "id" and e[2][1] == "savedregs":
            return "Ptr::NULL", "ptr1"
        if e[2][0] == "field" and show(e[2][1]) == "GC":
            return "Ptr::gc(%d)" % GCT[e[2][2]][1], "ptr4"
        return "/*TODO & %s*/" % show(e[2]), "?"
    # **(float **)(X) -> read through pointer at X
    d = deref(e)
    if d:
        a, p = d
        if a == "lvar":
            return p, LT[p]
        if a == "p":
            return "cx.ptr_at(%s)" % p, "ptr4"
        return "cx.%s(%s)" % (a, p), ATY[a]
    if k == "num":
        return numlit(e[1])
    if k == "str":
        return e[1], "str"
    if k == "id":
        nm = e[1]
        if nm in LT:
            return nm, LT[nm]
        if nm == "GC":
            return "GC", "gcptr"
        if nm == "this":
            return "Ptr::VE0", "ptr1"
        m = GLOB.match(nm)
        if m:
            a = {"dword": "i", "flt": "f", "byte": "b", "word": "i16"}.get(m.group(1), "i")
            return "cx.%s(Ptr::glob(0x%s))" % (a, m.group(2)), ATY[a]
        return nm, "?"
    if k == "field":
        if show(e[1]) == "GC":
            ty, off = GCT[e[2]]
            if ty == "expr":
                return "Ptr::gexpr(%d)" % off, "ptr4"
            return "cx.gc.%s" % e[2], {"af32": "af32", "ai32": "ai32"}.get(ty, ty)
        if show(e[1]) == "this" and e[2] == "history":
            return "Ptr::VE0.add(3560)", "ptr1"
        return "/*TODO field %s*/" % show(e), "?"
    if k == "idx":
        b, bt = emit(e[1])
        if bt in ("af32", "ai32"):
            return "%s[%s as usize]" % (b, emit_int(e[2])), bt[1:]
        return "/*TODO idx %s*/" % show(e), "?"
    if k == "cast":
        ty = e[1].replace(" ", "")
        inner = e[2]
        if ty == "float":
            if inner[0] == "bin" and inner[1] in "+-*/":
                a = emit_val(inner[2], "f32")
                b = emit_val(inner[3], "f32")
                return "(%s %s %s)" % (a, inner[1], b), "f32"
            s, t = emit(inner)
            return conv(s, t, "f32"), "f32"
        if ty == "double":
            return emit_val(inner, "f64"), "f64"
        if ty in ("int", "_DWORD", "unsignedint", "__int64", "char", "bool", "_BYTE", "unsigned__int8"):
            s, t = emit(inner)
            if is_ptr(t):
                return s, t   # pointer kept as pointer (int address)
            return conv(s, t, "i32"), "i32"
        if ty.endswith("*"):
            s, t = emit(inner)
            if t in ("ptr1", "ptr4", "?"):
                return s, ("ptr4" if tsize(ty) == 4 else "ptr1")
            if t in ("i32", "ilit") and s == "0":
                return "Ptr::NULL", "ptr1"
            return "/*?ptr*/(%s)" % s, ("ptr4" if tsize(ty) == 4 else "ptr1")
        return emit(inner)
    if k == "un":
        op = e[1]
        if op == "-":
            s, t = emit(e[2])
            if t in ("lit", "ilit"):
                return "-" + s, t
            return "(-%s)" % s, t
        if op == "!":
            return "!%s" % emit_val(e[2], "bool"), "bool"
        if op in ("++", "--"):
            d = deref(e[2])
            if d:
                return "cx.pre%s_%s(%s)" % ("inc" if op == "++" else "dec", d[0], d[1]), ATY[d[0]]
            n = lodw(e[2])
            if n is not None:
                return "cx.pre%s_i(Ptr::l(%d))" % ("inc" if op == "++" else "dec", 4 * n), "i32"
            nm = show(e[2])
            return "{ %s %s= 1; %s }" % (nm, "+" if op == "++" else "-", nm), LT.get(nm, "i32")
        if op == "~":
            return "!%s" % emit_int(e[2]), "i32"
        return "/*TODO %s*/" % show(e), "?"
    if k == "post":
        d = deref(e[2])
        if d:
            return "cx.post%s_%s(%s)" % ("inc" if e[1] == "++" else "dec", d[0], d[1]), ATY[d[0]]
        n = lodw(e[2])
        if n is not None:
            return "cx.post%s_i(Ptr::l(%d))" % ("inc" if e[1] == "++" else "dec", 4 * n), "i32"
        nm = show(e[2])
        return "{ let t = %s; %s %s= 1; t }" % (nm, nm, "+" if e[1] == "++" else "-"), LT.get(nm, "i32")
    if k == "bin":
        op = e[1]
        if op in ("&&", "||"):
            return "(%s %s %s)" % (emit_val(e[2], "bool"), op, emit_val(e[3], "bool")), "bool"
        a, ta = emit(e[2])
        b, tb = emit(e[3])
        if op in ("<", ">", "<=", ">=", "==", "!="):
            if is_ptr(ta) or is_ptr(tb):
                return "(%s %s %s)" % (conv(a, ta, "i32") if not is_ptr(ta) else a + ".bits()", op,
                                       conv(b, tb, "i32") if not is_ptr(tb) else b + ".bits()"), "bool"
            for w in ("f64", "f32"):
                if w in (ta, tb):
                    return "(%s %s %s)" % (conv(a, ta, w), op, conv(b, tb, w)), "bool"
            if "lit" in (ta, tb):
                return "(%s %s %s)" % (conv(a, ta, "f64"), op, conv(b, tb, "f64")), "bool"
            return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "bool"
        if is_ptr(ta) and op in "+-":
            sz = 4 if ta == "ptr4" else 1
            i = conv(b, tb, "i32")
            if op == "-":
                i = "-(%s)" % i
            return ("%s.add(%s)" % (a, i) if sz == 1 else "%s.add(4 * (%s))" % (a, i)), ta
        if is_ptr(tb) and op == "+":
            sz = 4 if tb == "ptr4" else 1
            i = conv(a, ta, "i32")
            return ("%s.add(%s)" % (b, i) if sz == 1 else "%s.add(4 * (%s))" % (b, i)), tb
        if ta in ("i32", "ilit", "bool", "i64") and tb in ("i32", "ilit", "bool", "i64"):
            if op in ("/", "%") or op in ("+", "-", "*"):
                if op in ("+", "-", "*"):
                    fn = {"+": "wrapping_add", "-": "wrapping_sub", "*": "wrapping_mul"}[op]
                    return "%s.%s(%s)" % (conv(a, ta, "i32") if ta != "ilit" else "(%s_i32)" % a, fn, conv(b, tb, "i32")), "i32"
            return "(%s %s %s)" % (conv(a, ta, "i32"), op, conv(b, tb, "i32")), "i32"
        # uncast float op -> x87 double
        return "(%s %s %s)" % (conv(a, ta, "f64"), op, conv(b, tb, "f64")), "f64"
    if k == "call":
        f = e[1]
        if f[0] == "id" and f[1] in ("LODWORD", "SLODWORD"):
            a, t = emit(e[2][0])
            if t == "f32":
                return "(%s.to_bits() as i32)" % a, "i32"
            if t == "f64":
                return "(%s as f32).to_bits() as i32" % a, "i32"
            return conv(a, t, "i32"), "i32"
        if f[0] == "id" and f[1] in CALLS:
            name, rt, ak = CALLS[f[1]]
            args = []
            for i, a in enumerate(e[2]):
                kind = ak[i] if ak and i < len(ak) else None
                if kind == "p":
                    args.append(emit_ptr(a)[0])
                elif kind == "f":
                    args.append(emit_val(a, "f32"))
                elif kind == "F":
                    args.append(emit_val(a, "f64"))
                elif kind == "i":
                    args.append(emit_val(a, "i32"))
                else:
                    s, t = emit(a)
                    args.append(conv(s, t, "f32") if t == "lit" else s)
            if name.startswith("vsmath::") or name == "i32::abs":
                return "%s(%s)" % (name, ", ".join(args)), rt
            if any("cx" in x for x in args):
                lets = "".join("let a%d = %s; " % (i, x) for i, x in enumerate(args))
                return "{ %s%s(cx%s) }" % (lets, name, "".join(", a%d" % i for i in range(len(args)))), rt
            return "%s(cx%s)" % (name, "".join(", " + x for x in args)), rt
        # car virtual call: (*(T **)(*(_DWORD *)(X) + K))(X)
        sf = show(f)
        m = re.search(r"\+(\d+)\)\)$", sf)
        if f[0] == "un" and m:
            return "car_vcall(cx, %s)" % m.group(1), "?"
        return "/*TODO call %s*/" % show(e), "?"
    if k == "tern":
        a, ta = emit(e[2])
        b, tb = emit(e[3])
        w = ta if ta == tb else ("f64" if "f64" in (ta, tb) else "f32" if "f32" in (ta, tb) else "i32")
        return "(if %s { %s } else { %s })" % (emit_val(e[1], "bool"), conv(a, ta, w), conv(b, tb, w)), w
    if k == "assign":
        return "{ %s }" % emit_assign(e[1], e[2], e[3]).strip(), "void"
    if k == "comma":
        return "{ %s; %s }" % (emit(e[1])[0], emit(e[2])[0]), emit(e[2])[1]
    return "/*TODO %s*/" % show(e), "?"

def emit_val(e, want):
    s, t = emit(e)
    return conv(s, t, want)

def emit_assign(op, l, r):
    n = lodw(l)
    if n is not None:
        s, t = emit(r)
        if is_ptr(t):
            PTRSLOTS.add(n)
            return "cx.lp[%d] = %s;" % (n, s)
        if op != "=":
            return "{ let t = cx.l.i(%d).%s(%s); cx.l.set_i(%d, t); }" % (4 * n, IOP[op], conv(s, t, "i32"), 4 * n)
        return "{ let t = %s; cx.l.set_i(%d, t); }" % (conv(s, t, "i32"), 4 * n)
    n = v132(l)
    if n is not None and n in PTRSLOTS and r[0] == "un" and r[1] == "*":
        x = r[2][2] if r[2][0] == "cast" else r[2]
        return "cx.lp[%d] = cx.ptr_at(%s);" % (n, emit_ptr(x)[0])
    if n is not None:
        s, t = emit(r)
        if op != "=":
            return "{ let t = (cx.l.f(%d) as f64 %s %s) as f32; cx.l.set_f(%d, t); }" % (4 * n, op[0], conv(s, t, "f64"), 4 * n)
        return "{ let t = %s; cx.l.set_f(%d, t); }" % (conv(s, t, "f32"), 4 * n)
    d = deref(l)
    if d:
        a, p = d
        s, t = emit(r)
        if a == "lvar":
            return "%s = %s;" % (p, conv(s, t, LT[p]))
        if op == "=":
            if a == "i" and t in ("f32",):
                return "{ let p = %s; let t = %s; cx.set_i(p, t.to_bits() as i32); }" % (p, s)
            if a == "p":
                return "{ let p = %s; let t = %s; cx.set_ptr(p, t); }" % (p, s)
            return "{ let p = %s; let t = %s; cx.set_%s(p, t); }" % (p, conv(s, t, ATY[a]), a)
        if a == "f":
            return "{ let p = %s; let t = (cx.f(p) as f64 %s %s) as f32; cx.set_f(p, t); }" % (p, op[0], conv(s, t, "f64"))
        return "{ let p = %s; let t = cx.%s(p).%s(%s); cx.set_%s(p, t); }" % (p, a, IOP[op], conv(s, t, "i32"), a)
    if l[0] == "id":
        nm = l[1]
        lt = LT.get(nm, "?")
        s, t = emit(r)
        if op == "=":
            return "%s = %s;" % (nm, conv(s, t, lt) if lt != "?" else s)
        if lt == "i32":
            return "%s = %s.%s(%s);" % (nm, nm, IOP[op], conv(s, t, "i32"))
        return "%s %s %s;" % (nm, op, conv(s, t, lt))
    return "/*TODO assign %s*/" % show(("assign", op, l, r))

IOP = {"+=": "wrapping_add", "-=": "wrapping_sub", "*=": "wrapping_mul", "/=": "wrapping_div", "|=": "bitor", "&=": "bitand", "^=": "bitxor"}

def emit_stmt(s, ind):
    k = s[0]
    o = []
    if k == "block":
        for x in s[1]:
            o += emit_stmt(x, ind)
    elif k == "lblock":
        o.append(ind + "'%s: {" % s[1])
        for x in s[2]:
            o += emit_stmt(x, ind + "    ")
        o.append(ind + "}")
    elif k == "expr":
        e = s[1]
        if e[0] == "assign":
            o.append(ind + emit_assign(e[1], e[2], e[3]))
        elif e[0] == "comma":
            o += emit_stmt(("expr", e[1]), ind)
            o += emit_stmt(("expr", e[2]), ind)
        else:
            r, t = emit(e)
            o.append(ind + ("let _ = " if t not in ("void", "?") and not r.startswith("{") else "") + r + ";")
    elif k == "if":
        o.append(ind + "if %s {" % emit_val(s[1], "bool"))
        o += emit_stmt(s[2], ind + "    ")
        if s[3] is not None:
            if s[3][0] == "if":
                sub = emit_stmt(s[3], ind)
                o.append(ind + "} else " + sub[0].lstrip())
                o += sub[1:]
                return o
            o.append(ind + "} else {")
            o += emit_stmt(s[3], ind + "    ")
        o.append(ind + "}")
    elif k == "for":
        if s[1]:
            o += emit_stmt(("expr", s[1]), ind)
        o.append(ind + "loop {")
        if s[2]:
            o.append(ind + "    if !%s { break; }" % emit_val(s[2], "bool"))
        body = emit_stmt(s[4], ind + "    ")
        if s[3]:
            step = emit_stmt(("expr", s[3]), ind + "    ")
            # continue must run the step: wrap body in a labeled block
            o.append(ind + "    'body: {")
            o += [("    " + x).replace("continue;", "break 'body;") for x in body]
            o.append(ind + "    }")
            o += step
        else:
            o += body
        o.append(ind + "}")
    elif k == "switch":
        o.append(ind + "match %s {" % emit_val(s[1], "i32"))
        pend = []
        for vals, st in s[2]:
            pend += vals
            if not st:
                continue
            body = list(st)
            if body and body[-1][0] == "break":
                body = body[:-1]
            elif not (body and body[-1][0] == "return"):
                o.append(ind + "    // TODO fallthrough")
            pat = " | ".join("_" if x == "default" else emit(x)[0] for x in pend)
            pend = []
            o.append(ind + "    %s => {" % pat)
            for x in body:
                o += emit_stmt(x, ind + "        ")
            o.append(ind + "    }")
        if pend:
            o.append(ind + "    %s => {}" % " | ".join("_" if x == "default" else emit(x)[0] for x in pend))
        if not any("default" in v for v, _ in s[2]):
            o.append(ind + "    _ => {}")
        o.append(ind + "}")
    elif k == "while":
        o.append(ind + "while %s {" % emit_val(s[1], "bool"))
        o += emit_stmt(s[2], ind + "    ")
        o.append(ind + "}")
    elif k == "do":
        o.append(ind + "loop {")
        o += emit_stmt(s[1], ind + "    ")
        o.append(ind + "    if !%s { break; }" % emit_val(s[2], "bool"))
        o.append(ind + "}")
    elif k == "return":
        o.append(ind + "return%s;" % ("" if s[1] is None else " " + emit(s[1])[0]))
    elif k in ("break", "continue"):
        o.append(ind + k + ";")
    elif k == "goto":
        o.append(ind + "break 'l_%s;" % s[1])
    elif k == "label":
        o.append(ind + "// %s:" % s[1])
    return o

# ---- goto -> labeled blocks (forward gotos only)
def contains_goto(s, lab):
    if s is None:
        return False
    if s[0] == "goto":
        return s[1] == lab
    for x in s[1:]:
        if isinstance(x, tuple) and contains_goto(x, lab):
            return True
        if isinstance(x, list) and any(isinstance(y, tuple) and contains_goto(y, lab) for y in x):
            return True
    return False

def fix_gotos(lst):
    i = 0
    while i < len(lst):
        s = lst[i]
        if s[0] == "label":
            lab = s[1]
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

for v in re.findall(r"\*\(float \*\)&(v\d+)", src):
    LT[v] = "f32"
PTRSLOTS = set(int(x) for x in re.findall(r"\*\)\s*\(?LODWORD\(v132\[(\d+)\]\)", src))
body = src[src.index("{"):]
body = "\n".join(l for l in body.split("\n")
                 if not re.match(r"^\s*(float|double|int|_DWORD|__int64|char|unsigned|_BYTE|bool|void|_UNKNOWN)\b[^=;(]*;", l)
                 and "memset(v132, 0xCCu" not in l)
body = re.sub(r"\((?:void|int|char|float|double|_DWORD \*|__int64) \(__thiscall \*\*\)\([^()]*\)\)", "(void **)", body)
body = re.sub(r"\((?:void|int|char|float|double|_DWORD \*|__int64) \(__thiscall \*\)\([^()]*\)\)", "(void *)", body)
p = P(tokenize(body))
try:
    top = p_stmt(p)
except Exception as ex:
    sys.stderr.write('%s near %s\n' % (ex, ' '.join(str(x[1]) for x in p.t[max(0,p.i-30):p.i+5])))
    raise SystemExit(1)
top = fix_tree(top)
# two passes so pointer slots discovered late are known everywhere
for _ in range(2):
    out = emit_stmt(top, "    ")
txt = "\n".join(out)
locs = []
for nm, t in sorted(LT.items(), key=lambda x: (len(x[0]), x[0])):
    if re.search(r"\b%s\b" % nm, txt):
        init = {"f32": "0.0f32", "f64": "0.0f64", "i32": "0i32", "i64": "0i64", "ptr1": "Ptr::NULL", "ptr4": "Ptr::NULL"}[t]
        locs.append("    let mut %s = %s;" % (nm, init))
print("    // generated by tools/phys_c2rs3.py; pointer slots %s" % sorted(PTRSLOTS))
print("\n".join(locs))
txt = txt.replace("cx.l.f(", "cx.lf(").replace("cx.l.i(", "cx.li(").replace("cx.l.set_f(", "cx.set_lf(").replace("cx.l.set_i(", "cx.set_li(")
txt = re.sub(r"cx\.lp\[(\d+)\] = (.*);$", lambda m: "{ let t = %s; cx.set_lpp(%s, t); }" % (m.group(2), m.group(1)), txt, flags=re.M)
txt = re.sub(r"cx\.lp\[(\d+)\]", r"cx.lpp(\1)", txt)
txt = txt.replace("Ptr::l(", "cx.lptr(")
print(txt)
