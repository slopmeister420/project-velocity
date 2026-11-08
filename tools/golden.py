"""golden-image gate for the performance stages (graphics; docs/graphics/details.md "Performance").

  golden.py capture <outdir> [--off x,y] [scene ...]         render the scenes (release build, hidden desktop)
  golden.py compare <refdir> <testdir> [--mean T] [--over P] [--diffs]
  golden.py perf <outdir> [--frames N] [--off envmap,road,mirror,stencil] [scene ...]
  golden.py list

Every run goes through tools/hidden_run.py, one game at a time. The exes are target/perf/release/{velocity,trackview}.exe
(override VELOCITY_PERF_DIR). Scenes run with VELOCITY_GOLDEN=<outdir> (visos_render::golden): frame-locked 1/60 s
time, a capture after each smoke-test shot once all render pipelines are compiled, time frozen while capturing.
Images are <outdir>/<scene>_<nn>.png. compare: per image max / mean abs error (0..255, RGB) and the % of pixels off
by more than 8; an image fails when mean > T (default 0.5) or % > P (default 0.2). Exit 1 on any failure or a
missing / extra image. perf: same scenes with VELOCITY_GOLDEN_PERF=N (default 300) and --fps 0 (no vsync), rows in
<outdir>/perf.csv and GPU pass timings in <outdir>/gpu.csv. Game scenes run with APPDATA = a fresh copy of the pinned
profile research/golden/profile_seed (options + saves, Rumble=1), so saved options cannot change the shots.
"""
import os
import struct
import subprocess
import sys
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = os.path.join(ROOT, ".venv-ida", "Scripts", "python.exe")
BIN = os.environ.get("VELOCITY_PERF_DIR", os.path.join(ROOT, "target", "perf", "release"))
GAME = os.environ.get("VELOCITY_GAME_DIR", "D:/games/xbox/games/project velocity debug")

# name: (exe, args, env). game scenes: one capture per smoke-test screenshot; trackview scenes: one capture.
SCENES = {
    # main menu, quick race, car select / flag girl, LADown dry race start + HUD, driving, braking, pause, quit
    "menus": ("velocity", ["--smoke-test"], {"SMOKE_DRIVE": "6"}),
    # London wet: rain, spray, wet road; the four car views (bumper / in-car with mirror) and look back
    "london_wet_cams": ("velocity", ["--smoke-test"], {"SMOKE_DIRECT": "1", "SMOKE_AREA": "2", "SMOKE_WET": "1",
                                                       "SMOKE_CAMS": "1", "SMOKE_DRIVE": "20"}),
    # Tokyo (Tokyo2 circuit) dry
    "tokyo": ("velocity", ["--smoke-test"], {"SMOKE_DIRECT": "1", "SMOKE_AREA": "3", "SMOKE_DRIVE": "8"}),
    # LA two-player split screen
    "split": ("velocity", ["--smoke-test"], {"SMOKE_SPLIT": "1", "SMOKE_DRIVE": "8"}),
    # career: profile select, plate, hangout (garage)
    "career": ("velocity", ["--smoke-test"], {"SMOKE_CAREER": "1", "SMOKE_DRIVE": "6"}),
    # particles / smoke / sparks / skids around two cars
    "tv_fx": ("trackview", ["--track", "ExecTest", "--particles", "--effects", "skid,smoke,sparks,slip,start",
                            "--fxat", "70,-13.4,133.6", "--cam", "70,-8,145,0,-25", "--size", "640x480"], {}),
    # night lights: lamp flares, road reflections, headlight beams, neon
    "tv_lights": ("trackview", ["--track", "LACoast", "--lightref", "--car", "LA/350z,LA/RX8", "--neon", "169,172",
                                "--caryaw", "0", "--size", "640x360", "--cam", "322,-55.6,-392,-45,-12"], {}),
    # rear-view mirror + damage-free car close up
    "tv_mirror": ("trackview", ["--track", "LADown", "--car", "LA/350z", "--size", "640x480",
                                "--cam", "197,-5,34,0,-8", "--mirror", "197,-5,14,180,-8"], {}),
    # split screen views with the zone ring / car stencil
    "tv_split_zone": ("trackview", ["--track", "LADown", "--car", "LA/350z", "--size", "800x600",
                                    "--cam", "197,-5,34,0,-8", "--split", "197,-5,14,180,-8"],
                      {"VELOCITY_ZONE_FORCE": "40"}),
    # front-end car select floor reflection and showroom car
    "tv_fe_carsel": ("trackview", ["--foyer", "carSel", "--fecam", "--car", "LA/350z", "--frontend",
                                   "--size", "640x480", "--cam", "5,1.4,0,0,-6"], {}),
    # hangout foyer characters
    "tv_hangout": ("trackview", ["--foyer", "hangout", "--size", "640x480", "--cam", "-0.4,1.5,0.6,90,-8"], {}),
}


SEED = os.path.join(ROOT, "research", "golden", "profile_seed", "Velocity")


def pinned_profile(outdir):
    """a fresh copy of the pinned profile (options.ini with Rumble=1 + saves) under <outdir>/_profile, used as
    APPDATA: the smoke run writes its options back, which made the pause menu's Rumble ON/OFF differ run to run"""
    import shutil
    if not os.path.isdir(SEED):
        src = os.path.join(os.environ.get("APPDATA", ""), "Velocity")
        shutil.copytree(src, SEED) if os.path.isdir(src) else os.makedirs(SEED)
        ini = os.path.join(SEED, "options.ini")
        lines = open(ini).read().splitlines() if os.path.isfile(ini) else ["; Project Velocity options"]
        lines = [l for l in lines if not l.startswith("Rumble=")] + ["Rumble=1"]
        open(ini, "w").write("\n".join(lines) + "\n")
    prof = os.path.join(outdir, "_profile")
    shutil.rmtree(prof, ignore_errors=True)
    shutil.copytree(SEED, os.path.join(prof, "Velocity"))
    return prof


def run_scene(name, outdir, extra_env, extra_args, timeout=600):
    exe, args, env = SCENES[name]
    path = os.path.join(BIN, exe + ".exe")
    cmd = [PY, os.path.join(ROOT, "tools", "hidden_run.py"), "--timeout", str(timeout)]
    allenv = dict(env)
    allenv.update({"VELOCITY_GOLDEN": outdir, "VELOCITY_GOLDEN_PREFIX": name, "SMOKE_PREFIX": "golden_" + name})
    if exe == "velocity":
        allenv["APPDATA"] = pinned_profile(outdir)
    allenv.update(extra_env)
    for k, v in allenv.items():
        cmd += ["--env", "%s=%s" % (k, v)]
    shot = [] if exe == "velocity" else ["--shot", os.path.join(outdir, name + "_tv.png")]
    cmd += ["--", path, "--game", GAME] + args + shot + extra_args
    log = os.path.join(outdir, name + ".log")
    for p in os.listdir(outdir):  # stale images from an earlier run would fake a "missing" in compare
        if p.startswith(name + "_") and p.endswith(".png"):
            os.remove(os.path.join(outdir, p))
    with open(log, "w") as f:
        r = subprocess.run(cmd, cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)
    n = len([p for p in os.listdir(outdir) if p.startswith(name + "_") and p.endswith(".png")])
    print("%-16s exit %d, %d images" % (name, r.returncode, n), flush=True)


def scenes_from(rest):
    names = [a for a in rest if not a.startswith("--")] or list(SCENES)
    for n in names:
        if n not in SCENES:
            sys.exit("unknown scene " + n)
    return names


def opt(rest, name, default):
    return rest[rest.index(name) + 1] if name in rest else default


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", path
    pos, idat, w = 8, [], 0
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if kind == b"IHDR":
            w, h, depth, ctype = struct.unpack(">IIBB", body[:10])
            assert depth == 8 and ctype in (2, 6), (path, depth, ctype)
            bpp = 3 if ctype == 2 else 4
        elif kind == b"IDAT":
            idat.append(body)
        pos += 12 + n
    raw = zlib.decompress(b"".join(idat))
    stride = w * bpp
    out = bytearray(h * stride)
    prev = bytearray(stride)
    for y in range(h):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        if f == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 255
        elif f == 2:
            line = bytearray((a + b) & 255 for a, b in zip(line, prev))
        elif f == 3:
            for i in range(stride):
                left = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        out[y * stride:(y + 1) * stride] = line
        prev = line
    # to RGB
    if bpp == 4:
        rgb = bytearray(w * h * 3)
        rgb[0::3], rgb[1::3], rgb[2::3] = out[0::4], out[1::4], out[2::4]
        out = rgb
    return w, h, bytes(out)


def write_png(path, w, h, rgb):
    raw = b"".join(b"\x00" + rgb[y * w * 3:(y + 1) * w * 3] for y in range(h))
    chunk = lambda k, b: struct.pack(">I", len(b)) + k + b + struct.pack(">I", zlib.crc32(k + b) & 0xFFFFFFFF)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))


def compare(refdir, testdir, mean_tol, over_tol, diffs):
    pngs = lambda d: sorted(p for p in os.listdir(d) if p.endswith(".png") and not p.startswith("diff_"))
    ref, test = pngs(refdir), pngs(testdir)
    bad = 0
    for p in sorted(set(ref) | set(test)):
        if p not in ref or p not in test:
            print("%-28s MISSING in %s" % (p, "ref" if p not in ref else "test"))
            bad += 1
            continue
        w, h, a = read_png(os.path.join(refdir, p))
        w2, h2, b = read_png(os.path.join(testdir, p))
        if (w, h) != (w2, h2):
            print("%-28s SIZE %dx%d vs %dx%d" % (p, w, h, w2, h2))
            bad += 1
            continue
        d = bytes(abs(x - y) for x, y in zip(a, b))
        mx, mean = max(d), sum(d) / len(d)
        over = sum(1 for i in range(0, len(d), 3) if max(d[i], d[i + 1], d[i + 2]) > 8) * 100.0 / (w * h)
        ok = mean <= mean_tol and over <= over_tol
        bad += not ok
        print("%-28s max %3d mean %7.4f over8 %6.3f%% %s" % (p, mx, mean, over, "ok" if ok else "FAIL"))
        if diffs and mx:
            write_png(os.path.join(testdir, "diff_" + p), w, h, bytes(min(255, v * 8) for v in d))
    print("%d images, %d failed" % (len(set(ref) | set(test)), bad))
    return bad


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("capture", "compare", "perf", "list"):
        sys.exit(__doc__)
    cmd, rest = sys.argv[1], sys.argv[2:]
    if cmd == "list":
        for n, (exe, args, env) in SCENES.items():
            print(n, exe, " ".join(args), env)
        return
    if cmd == "compare":
        sys.exit(1 if compare(rest[0], rest[1], float(opt(rest, "--mean", "0.5")), float(opt(rest, "--over", "0.2")),
                              "--diffs" in rest) else 0)
    outdir = os.path.abspath(rest[0])
    os.makedirs(outdir, exist_ok=True)
    skip = {opt(rest, k, None) for k in ("--frames", "--off")}
    names = scenes_from([a for a in rest[1:] if a not in skip])
    for n in names:
        if cmd == "capture":
            run_scene(n, outdir, {"VELOCITY_PERF_OFF": opt(rest, "--off", "")} if "--off" in rest else {}, [])
        else:
            env = {"VELOCITY_GOLDEN_PERF": opt(rest, "--frames", "300")}
            if "--off" in rest:
                env["VELOCITY_PERF_OFF"] = opt(rest, "--off", "")
            run_scene(n, outdir, env, ["--fps", "0"] if SCENES[n][0] == "velocity" else [])


if __name__ == "__main__":
    main()
