# physics agent: generate the CGlobalConfig struct + loader body from research/phys_dump/gcfg_sorted.txt
import re
rows = []
for line in open(r"D:\projects\project poland rewrite\research\phys_dump\gcfg_sorted.txt"):
    kind = line.split()[0]
    parts = [p.strip() for p in line.split("|")]
    name = re.search(r'"([^"]+)"', line).group(1)
    if name == "stop_loading":
        continue
    off = parts[1].replace("v8[820]", "").replace("+", "").strip()
    off = int(off) if off else 0
    if kind == "EXPR":
        rows.append((off, kind, 1, name, parts[3].strip('"')))
    else:
        rows.append((off, kind, int(parts[2]), name, None))
rows.sort()
out_struct = []
out_load = []
prev_end = 0
for off, kind, cnt, name, vars_ in rows:
    if off != prev_end:
        out_struct.append("    // gap 0x%X..0x%X" % (prev_end, off))
    if kind == "EXPR":
        ty, size = "Expression", 4
        out_load.append('        self.%s = r.read_expression("%s", "%s")?;' % (name, name, vars_))
    elif kind == "REAL":
        ty = "f32" if cnt == 1 else "[f32; %d]" % cnt
        size = 4 * cnt
        out_load.append('        r.read_reals(%sself.%s, "%s")?;' % ("std::slice::from_mut(&mut " if cnt == 1 else "&mut ", name, name) if False else
                        ('        r.read_real(&mut self.%s, "%s")?;' % (name, name) if cnt == 1 else '        r.read_reals(&mut self.%s, "%s")?;' % (name, name)))
    else:
        ty = "i32" if cnt == 1 else "[i32; %d]" % cnt
        size = 4 * cnt
        out_load.append('        r.read_int(&mut self.%s, "%s")?;' % (name, name) if cnt == 1 else '        r.read_ints(&mut self.%s, "%s")?;' % (name, name))
    out_struct.append("    pub %s: %s, // +0x%X" % (name, ty, off))
    if name == "PL_omega_scale_z":
        out_load.append("        self.PL_omega_scale_mode = if self.PL_omega_scale_x == 1.0 && self.PL_omega_scale_y == 1.0 && self.PL_omega_scale_z == 1.0 { 0.0 } else { 2.0 };")
    prev_end = off + size
out_struct.append("    /// set after `PL_omega_scale_z`: 0 when all three scales are 1, else 2")
out_struct.append("    pub PL_omega_scale_mode: f32, // +0x%X" % prev_end)
open(r"D:\projects\project poland rewrite\research\phys_dump\gen_global.rs", "w").write(
    "\n".join(out_struct) + "\n\n// ---- load\n" + "\n".join(out_load) + "\n")
print(len(rows), "end", hex(prev_end))
