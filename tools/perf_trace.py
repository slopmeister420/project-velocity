"""summarise a bevy trace_chrome json (one event per line) into total / per-frame CPU time per span.

  perf_trace.py <trace.json> [top N] [--from SEC] [--to SEC]

Spans are keyed by name plus args.name (bevy's `system{name=...}`, schedule names, render graph nodes); B/E pairs are
matched per thread, X events use dur. Frames = count of the "frame" span (bevy's main-loop span). Times are inclusive,
so schedules contain their systems; the main thread and the render thread overlap (pipelined rendering).
--from skips events before SEC seconds into the trace (loading).
"""
import json
import sys
from collections import defaultdict


def key(ev):
    a = ev.get("args") or {}
    n = ev.get("name", "?")
    for k in ("name", "system", "schedule", "label", "node"):
        if k in a and isinstance(a[k], str):
            return "%s{%s}" % (n, a[k])
    return n


def main():
    path = sys.argv[1]
    top = int(sys.argv[2]) if len(sys.argv) > 2 and not sys.argv[2].startswith("--") else 40
    t0 = float(sys.argv[sys.argv.index("--from") + 1]) * 1e6 if "--from" in sys.argv else 0.0
    t1 = float(sys.argv[sys.argv.index("--to") + 1]) * 1e6 if "--to" in sys.argv else float("inf")
    total = defaultdict(float)
    count = defaultdict(int)
    stacks = defaultdict(list)
    first = None
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip().rstrip(",")
            if not line.startswith("{"):
                continue
            try:
                ev = json.loads(line)
            except ValueError:
                continue
            ts = float(ev.get("ts", 0))
            if first is None:
                first = ts
            ph = ev.get("ph")
            tid = (ev.get("pid"), ev.get("tid"))
            if ph == "B":
                stacks[tid].append((key(ev), ts))
            elif ph == "E":
                if stacks[tid]:
                    k, start = stacks[tid].pop()
                    if t0 <= start - first < t1:
                        total[k] += ts - start
                        count[k] += 1
            elif ph == "X":
                if t0 <= ts - first < t1:
                    k = key(ev)
                    total[k] += float(ev.get("dur", 0))
                    count[k] += 1
    # bevy 0.19 names the main-loop span "update"
    frames = max((c for k, c in count.items() if k.strip().rstrip(":") in ("frame", "update")), default=1) or 1
    print("frames %d" % frames)
    print("%10s %10s %8s  %s" % ("total_ms", "ms/frame", "calls", "span"))
    for k, v in sorted(total.items(), key=lambda kv: -kv[1])[:top]:
        print("%10.1f %10.4f %8d  %s" % (v / 1000, v / 1000 / frames, count[k], k))


if __name__ == "__main__":
    main()
