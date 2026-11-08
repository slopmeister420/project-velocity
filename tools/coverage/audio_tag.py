"""tag the unported entries of research/audio_coverage.txt: python tools/coverage/audio_tag.py
(run tools/coverage/cov.py audio research/audio_names.tsv first)
"""
import re, os
from collections import Counter
root = r"D:\projects\project poland rewrite\research"
p = os.path.join(root, "audio_coverage.txt")
lines = open(p).read().splitlines()
PLATFORM = re.compile(r"_AUDIO_(MixDown|Resample|SoftwareMix|StreamRead|UpdateSoftwareMix|SetEnvironmentM|InitM|Lock|Unlock|"
                      r"Log10|ReadBytes|IsSoftwareSample|UnloadBankDataM|Alloc|Free)|AUDIO_SetMemoryCallbacks|"
                      r"VsSoundtrackPlayer::(DecodePacket|SubmitPacket|FindFreePacket|PacketDone|VerifySongs)|"
                      r"AUDIO_QueueSample__|AUDIO_SetListenerVelocity__|VsSoundtrack::OpenSong")
CTOR = re.compile(r"VsDestructor|VsArray::|VsVector4|VsHashedString|VsEnvSound::VsEnvSound|VectorDeletingDtor|"
                  r"~|VsAudioVehics__|VsBaseCamera::GetPositionInline")
out = []
for l in lines:
    if not l.startswith("0x"):
        if not l.startswith("# unported by"):
            out.append(l)
        continue
    a = l.split("\t")
    nm, sz = a[3], int(a[1])
    if PLATFORM.search(nm):
        tag = "platform(replaced by the Rust mixer/pools/wma)"
    elif CTOR.search(nm):
        tag = "ctor-dtor"
    elif re.match(r"(TimelineSound|TimelineGlobalAudio)__", nm):
        tag = "ctor-dtor(timeline object boilerplate; handlers ported in velocity_game)"
    elif re.search(r"::(UpdateStub|UpdateZone|SetFlag40|UpdateFlag40|GetState|GetSoundtrack|GetSongInfo|GetSongCount|"
                   r"GetListName|SetUnused|GetSoundtrackCount|SetType|SetShuffle)", nm) \
            or re.search(r"AUDIO_Get(ActiveEventCount|Max|Free|Alloc)", nm) or sz < 48:
        tag = "getter(trivial)"
    else:
        tag = "audio-behaviour"
    out.append("\t".join(a[:4] + [tag]) + ("\tmentioned" if l.endswith("mentioned") else ""))
c, cb = Counter(), Counter()
for l in out:
    if l.startswith("0x"):
        t = l.split("\t")[4].split("(")[0]
        c[t] += 1
        cb[t] += int(l.split("\t")[1])
hdr = "# unported by category (count/bytes): " + "; ".join("%s %d/%d" % (k, c[k], cb[k]) for k in sorted(cb, key=lambda k: -cb[k]))
i = next(n for n, l in enumerate(out) if l.startswith("0x"))
out.insert(i, hdr)
open(p, "w").write("\n".join(out) + "\n")
print(hdr)
