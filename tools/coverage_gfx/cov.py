"""graphics coverage: functions in graphics regions vs '/// File.cpp 0x...' origin lines in crates/visos_render.
usage: python tools/coverage_gfx/cov.py   -> research/graphics_coverage.txt"""
import re, os
from collections import Counter

root = r"D:\projects\project poland rewrite"
PFX = ("Render\\", "Renderer\\", "elParticle\\", "StatelessParticles\\", "VertexAnimation\\", "BoneAnimation\\")
SPEC = ("src\\Core\\VsRender.cpp", "src\\ParticleManager\\")
regs = []
for l in open(os.path.join(root, "research", "objregions.tsv")):
    p = l.rstrip("\r\n").split("\t")
    if len(p) < 3: continue
    f = p[2]
    if f.startswith(PFX) or f.startswith(SPEC):
        regs.append((int(p[0], 16), int(p[1], 16), f))
regs.sort()
names = {}
for fn in ("ida_rebuild_names.tsv", "ida_salvaged_names.tsv"):
    pth = os.path.join(root, "research", fn)
    if not os.path.exists(pth): continue
    for l in open(pth, encoding="utf-8", errors="replace"):
        p = l.rstrip("\r\n").split("\t")
        if len(p) >= 2:
            try: names.setdefault(int(p[0], 16), p[1])
            except ValueError: pass
funcs = []
for l in open(os.path.join(root, "research", "funcmap.tsv")):
    p = l.rstrip("\r\n").split("\t")
    a = int(p[0], 16); sz = int(p[1])
    for s, e, f in regs:
        if s <= a < e:
            funcs.append((a, sz, f)); break
orig = set(); anyhex = set()
for dp, dn, fn in os.walk(os.path.join(root, "crates", "visos_render", "src")):
    for n in fn:
        if not n.endswith(".rs"): continue
        for l in open(os.path.join(dp, n), encoding="utf-8", errors="replace"):
            s = l.strip()
            if s.startswith("//"):
                if re.search(r"\w\.cpp\s+0x[0-9A-Fa-f]+", s):
                    m = re.match(r"///?\s*\S+\.cpp\s+(0x[0-9A-Fa-f]+(\s*[/,+]\s*0x[0-9A-Fa-f]+)*)", s)
                    if m:
                        for h in re.findall(r"0x([0-9A-Fa-f]+)", m.group(1)): orig.add(int(h, 16))
            for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s): anyhex.add(int(h, 16))

OVR = {0x13E530: "debug", 0x11E9F0: "debug", 0x102FD0: "debug", 0x144DF0: "debug", 0x145A80: "debug", 0x145620: "debug",
       0x2D6700: "debug",  # CType save (editor), not rendering
       0x1044D0: "mis-attributed?", 0x105370: "mis-attributed?",
       0x2C7570: "template-dup", 0x2C80F0: "template-dup", 0x2CBB10: "template-dup",
       0x2C9380: "template-dup", 0x2C9B80: "template-dup", 0x2CA670: "template-dup"}  # from IDA (tools/ida/gfx_cov_top.py)
RLOGIC = {"ShaderManagerBase.cpp", "ShaderManager.cpp", "Renderer.cpp", "DrawList.cpp", "ShaderProgramManager.cpp",
          "VISOS_ColourXBOX.cpp"}
def tag(a, sz, f):
    if a in OVR: return OVR[a]
    nm = names.get(a, "")
    # Renderer\ = the ViSOS D3D layer. Pure device/resource wrappers (vertex/index buffers, texture and target
    # objects, matrix stacks, push-buffer prims) are replaced by Bevy/wgpu; the files whose logic decides what a
    # draw looks like (stage/blend state, draw order, program binding, maths helpers) are "renderer-logic".
    # Render\Pc_xbox\*_PC_XBOX, elParticleRenderer_Xbox, StatelessParticleRenderInst_XBOX are game code.
    base = f.split("\\")[-1]
    if f.startswith("Renderer\\"):
        if base in RLOGIC: return "renderer-logic"
        return "platform-replaced"
    stem = os.path.basename(f.replace("\\", "/"))[:-4]
    if re.search(r"Debug|Tweak|Dump|Print|Stats?\b|Trace|Profil|Log", nm): return "debug"
    m = re.match(r"(?:.*::)?(\w+)::(~?)(\w+)$", nm)
    if m and (m.group(3) == m.group(1) or m.group(2) == "~"): return "ctor-dtor"
    if re.search(r"::(Get|Is|Has|Set)[A-Z]", nm) and sz < 120: return "getter"
    if sz < 40: return "getter"
    if m and m.group(1) not in stem and stem not in m.group(1) and not f.startswith("Renderer"):
        return "mis-attributed?"
    return "visible-rendering"

port = [x for x in funcs if x[0] in orig]
miss = [x for x in funcs if x[0] not in orig]
men = [x for x in miss if x[0] in anyhex]
tb = sum(x[1] for x in funcs); pb = sum(x[1] for x in port)
out = ["# graphics coverage: functions in graphics regions (objregions.tsv + funcmap.tsv) vs '/// File.cpp 0x...' origin lines in crates/visos_render",
       "# ported %d/%d functions, %d/%d bytes (%.1f%%); regions %d" % (len(port), len(funcs), pb, tb, 100.0 * pb / tb, len(regs)),
       "# plus %d unported-by-origin-line functions whose address appears in a code comment (mentioned), %d B" % (len(men), sum(x[1] for x in men)),
       "# unported: addr size file name tag [mentioned = address appears in a comment but has no origin line], largest first"]
c = Counter(); cb = Counter(); rows = []
for a, sz, f in sorted(miss, key=lambda x: -x[1]):
    t = tag(a, sz, f); c[t] += 1; cb[t] += sz
    rows.append("0x%08X\t%d\t%s\t%s\t%s%s" % (a, sz, f.split("\\")[-1], names.get(a, ""), t, "\tmentioned" if a in anyhex else ""))
out.append("# tags: " + ", ".join("%s %d fn/%d B" % (k, c[k], cb[k]) for k in c))
pt = sum(sz for a, sz, f in funcs if tag(a, sz, f) == "platform-replaced")
pp = sum(sz for a, sz, f in port if tag(a, sz, f) == "platform-replaced")
out.insert(2, "# excluding platform-replaced files (D3D wrappers Bevy replaces): %d/%d bytes (%.1f%%)" % (pb - pp, tb - pt, 100.0 * (pb - pp) / (tb - pt)))
# per-file ported/total
pf = Counter(); tf = Counter(); pfb = Counter(); tfb = Counter()
for a, sz, f in funcs:
    k = f.split("\\")[-1]; tf[k] += 1; tfb[k] += sz
    if a in orig: pf[k] += 1; pfb[k] += sz
out.append("# per file (ported/total fn, ported/total bytes):")
for k in sorted(tf, key=lambda k: -(tfb[k] - pfb[k])):
    out.append("#   %-40s %d/%d  %d/%d" % (k, pf[k], tf[k], pfb[k], tfb[k]))
out += rows
open(os.path.join(root, "research", "graphics_coverage.txt"), "w").write("\n".join(out) + "\n")
print(out[1]); print(out[3])
