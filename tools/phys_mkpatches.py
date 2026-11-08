# physics agent: builds tools/phys_c2rs4_patches.txt (source-level rewrites applied by phys_c2rs4.py
# before translation) from the current decompiler dumps. Re-run after re-dumping any function below.
import re, glob

ROOT = r"D:\projects\project poland rewrite"
FUNCS = {}
files = sorted(glob.glob(ROOT + r"\research\phys_dump\*.c"), key=lambda f: f.endswith("Redump.c"))
for f in files:
    t = open(f, encoding="utf-8", errors="replace").read()
    parts = re.split(r"(?m)^// ==== ([0-9A-F]{8}) (\S+)\n", t)
    for i in range(1, len(parts), 3):
        a = int(parts[i], 16)
        if f.endswith("Redump.c") or a not in FUNCS:
            FUNCS[a] = parts[i + 2]

out = []

def patch(addr, old, new):
    assert FUNCS[addr].count(old) == 1, (hex(addr), old[:80])
    out.append("=== %X\n<<<\n%s\n---\n%s\n>>>\n" % (addr, old.rstrip("\n"), new.rstrip("\n")))

def between(addr, a, b, inclusive_b=False):
    f = FUNCS[addr]
    i = f.index(a)
    j = f.index(b, i)
    return f[i:j + (len(b) if inclusive_b else 0)]

# --- 1D1850 VehicleEntity::Update: the gearbox block jumps between two identical branches with
# gotos (LABEL_396/397/398); structured equivalent
f = FUNCS[0x1D1850]
old = between(0x1D1850, "        if ( *(int *)(LODWORD(v132[247]) + 3216) <= 90 )\n        {\n          if ( *(_DWORD *)(LODWORD(v132[246]) + 160) == 0 )\n            goto LABEL_397;",
              "LABEL_398:\n", True)
tail = """        if ( *(int *)(LODWORD(v132[247]) + 3216) <= 90 )
        {
          if ( *(_DWORD *)(LODWORD(v132[246]) + 160) == 0 )
            AUDIO_Resample__35D4B0(
              (int *)(LODWORD(v132[247]) + 240),
              v37,
              *(float *)(LODWORD(v132[246]) + 36),
              *(_DWORD *)(*(_DWORD *)(LODWORD(v132[247]) + 3552) + 108));
          else
            AUDIO_Resample__35D4B0((int *)(LODWORD(v132[247]) + 240), v37, *(float *)(LODWORD(v132[246]) + 36), 0);
        }
        else
        {
          *(_DWORD *)(LODWORD(v132[247]) + 3216) -= 100;
          PhysicsCarInput__1BA170((_DWORD *)(LODWORD(v132[247]) + 1444), 0);
        }
"""
k = old.index("      else\n      {\n")
m = old.index("        if ( *(int *)(LODWORD(v132[247]) + 3216) <= 90 )", k)
patch(0x1D1850, old, tail + "      }\n" + old[k:m] + tail + "      }\n")

# --- 1C9070 VehicleEntity::LoadConfig: Main.cfg is read by c_config (Rust) before this runs
patch(0x1C9070, between(0x1C9070, "  v22[446] = dword_6CDCE0;", "  if ( *(float *)(v22[445] + 4096) != 0.0 )"),
      "  v22[445] = this;\n  v22[444] = 2;\n  v22[443] = 1;\n")

# --- 1CABF0 VehicleEntity::Create: spans and performance modifiers come from the caller (model
# locators and vehicle setup belong to the vehicle database); the local config is loaded by Rust
F = re.search(r"memset\((\w+), 0xCCu", FUNCS[0x1CABF0]).group(1)
patch(0x1CABF0, between(0x1CABF0, "  %s[18] = VsAIRaceVehicle__23F40(" % F, "  v22 = a7;"), "")
patch(0x1CABF0, between(0x1CABF0, "  if ( *(_DWORD *)(%s[19] + 3552) == 0 )" % F, "  VsAIVehicle__2C6D0((_DWORD *)%s[19], a3);" % F),
      "  VehicleEntity__1C9070((VehicleEntity *)%s[19], v22, (int)\"Main.cfg\", dword_75B28C);\n" % F)

# --- 3682B0 CFastCar::SuggestedDynamicVelocity: `goto LABEL_35` leaves the settle loop into the else
# branch; same control flow with a flag
f = FUNCS[0x3682B0]
F = re.search(r"memset\((\w+), 0xCCu", f).group(1)
patch(0x3682B0, "  memset(%s, 0xCCu, sizeof(%s));" % (F, F),
      "  int goto35; // patch\n  memset(%s, 0xCCu, sizeof(%s));\n  goto35 = 0;" % (F, F))
patch(0x3682B0, "          goto LABEL_35;", "          goto35 = 1;\n          break;")
avg = between(0x3682B0, "      %s[127] = %s[40] / (float)SLODWORD(%s[39]);" % (F, F, F), "    }\n    else\n    {\nLABEL_35:")
patch(0x3682B0, avg + "    }\n    else\n    {\nLABEL_35:",
      "      if ( goto35 != 0 )\n        %s[127] = -1.0;\n      else\n      {\n%s      }\n    }\n    else\n    {\n" % (F, avg))

open(ROOT + r"\tools\phys_c2rs4_patches.txt", "w", encoding="utf-8").write("\n".join(out))
print("patches:", len(out))
