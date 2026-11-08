"""coverage audit: original functions in a subsystem's source files vs '/// File.cpp 0x...' origin lines.

usage: python tools/coverage/cov.py <subsystem> [names.tsv]     (subsystem: ai | core)
writes research/<subsystem>_coverage.txt ; names.tsv = addr<TAB>name (optional, from ida_names.py)
"""
import re, os, sys

root = r"D:\projects\project poland rewrite"
sub = sys.argv[1]

CORE_DIRS = ("Camera", "Cli", "EventDatabase", "GameBridge", "GameDataLoading", "GameInput", "GameMode",
             "Message", "Replays", "ScoreManager", "TimelineManager", "Triggers", "Controller")
CORE_FILES = ("src\\main.cpp", "src\\VsCommonMain.cpp", "InputManager.cpp", "InputBinding.cpp",
              "InputMapping.cpp", "InputDevice.cpp")
CORE_VISOS = ("Memory\\", "File\\", "Handy\\", "Timer\\", "Instances\\", "Compartment\\", "Type\\", "Chunk\\",
              "Timeline\\")


def is_sub(f):
    if sub == "ai":
        return ("\\Ai\\" in f) or "OpponentDatabase" in f or "TeamDatabase" in f
    if sub == "core":
        if f in CORE_FILES:
            return True
        if f.startswith("src\\"):
            return f.split("\\")[1] in CORE_DIRS and "VsAITrafficLight" not in f
        return f.startswith(CORE_VISOS) or f in CORE_FILES
    if sub == "audio":
        # objregions mis-attributes the visos Audio layer (0x2B9A00.., 0x311EC0..), so the scope is
        # research/audio_names.tsv (tools/ida/audio_cov_names.py: every *AUDIO_* function + src/Audio regions)
        return False
    raise SystemExit("unknown subsystem")


CRATES = {"ai": ["velocity_ai"], "core": ["velocity_game", "visos", "velocity"], "audio": ["velocity_audio"]}
AUDIO_SCOPE = {}
if sub == "audio":
    for l in open(os.path.join(root, "research", "audio_names.tsv")):
        p = l.rstrip("\n").split("\t")
        a = int(p[0], 16)
        # XDK/mis-attributed neighbours of the Resample region (physics etc. from 0x355C40 on)
        if p[5] == "visos" and "__" in p[1] and a >= 0x355C40 and "_AUDIO_InitM" not in p[1]:
            continue
        AUDIO_SCOPE[a] = p[1]
OWN = list(CRATES[sub])
if sub == "audio":
    # TimelineSound / TimelineGlobalAudio are ported in velocity_game (core): their origin lines count too
    CRATES["audio"] = ["velocity_audio", "velocity_game"]
# core-scope functions ported by another owner (e.g. GameDataLoading's model fixups in visos_render) count too
if sub == "core":
    CRATES["core"] += [d for d in sorted(os.listdir(os.path.join(root, "crates")))
                       if d not in CRATES["core"] and os.path.isdir(os.path.join(root, "crates", d, "src"))]

regs = []
for l in open(os.path.join(root, "research", "objregions.tsv")):
    p = l.rstrip("\r\n").split("\t")
    if len(p) < 3:
        continue
    if is_sub(p[2]):
        regs.append((int(p[0], 16), int(p[1], 16), p[2]))
funcs = []
for l in open(os.path.join(root, "research", "funcmap.tsv")):
    p = l.rstrip("\r\n").split("\t")
    a = int(p[0], 16)
    sz = int(p[1])
    if a in AUDIO_SCOPE:
        funcs.append((a, sz, "Audio\\" + AUDIO_SCOPE[a]))
    for s, e, f in regs:
        if s <= a < e:
            funcs.append((a, sz, f))
orig = set()
anyhex = set()
for crate in CRATES[sub]:
    for dp, dn, fn in os.walk(os.path.join(root, "crates", crate, "src")):
        for n in fn:
            if not n.endswith(".rs"):
                continue
            for l in open(os.path.join(dp, n), encoding="utf-8", errors="replace"):
                s = l.strip()
                if s.startswith("//") and re.match(r"//[/!]?\s*\w+(\.cpp)?\s+0x[0-9A-Fa-f]+", s):
                    m = re.match(r"//[/!]?\s*\w+(?:\.cpp)?\s+(0x[0-9A-Fa-f]+(\s*[/,+&]\s*0x[0-9A-Fa-f]+)*)", s)
                    if m:
                        for h in re.findall(r"0x([0-9A-Fa-f]+)", m.group(1)):
                            orig.add(int(h, 16))
                    # the owner's crates: every address on an origin line; another owner's: only the leading list
                    if crate in OWN:
                        for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s):
                            orig.add(int(h, 16))
                for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s):
                    anyhex.add(int(h, 16))
names = {}
if len(sys.argv) > 2 and os.path.exists(sys.argv[2]):
    for l in open(sys.argv[2]):
        p = l.rstrip("\n").split("\t")
        if len(p) >= 2:
            names[int(p[0], 16)] = p[1]
tot = len(funcs)
totb = sum(f[1] for f in funcs)
port = [f for f in funcs if f[0] in orig]
miss = [f for f in funcs if f[0] not in orig]
pb = sum(f[1] for f in port)
out = []
out.append("# %s coverage: functions in %s regions (objregions.tsv) vs '/// File.cpp 0x...' origin lines in crates %s"
           % (sub, sub, ",".join(CRATES[sub])))
out.append("# ported %d/%d functions, %d/%d bytes (%.1f%%)" % (len(port), tot, pb, totb, 100.0 * pb / max(totb, 1)))
out.append("# unported (addr size file name [mentioned = address appears in a comment but has no origin line]), largest first")
for a, sz, f in sorted(miss, key=lambda x: -x[1]):
    out.append("0x%08X\t%d\t%s\t%s%s" % (a, sz, f.split("\\")[-1], names.get(a, ""),
                                          "\tmentioned" if a in anyhex else ""))
open(os.path.join(root, "research", sub + "_coverage.txt"), "w").write("\n".join(out) + "\n")
print(out[1])
