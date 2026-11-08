# name the TrailBlazerRender / danger particle functions found this stage
import idc
N = {
    0x14DEF0: "TrailBlazerRender__Ctor",
    0x14DF70: "TrailBlazerRender__Init",
    0x14E2A0: "TrailBlazerRender__Reset",
    0x14E360: "TrailBlazerRender__Shutdown",
    0x14E480: "TrailBlazerRender__AddSectionTrails",
    0x14E530: "TrailBlazerRender__Update",
    0x14E710: "TrailBlazerRender__Render",
    0x14E940: "TrailBlazerRender__RenderGrowing",
    0x14F3A0: "TrailBlazerRender__RenderShown",
    0x14F960: "TrailBlazerRender__RenderFading",
    0x150140: "TrailBlazerRender__GroundRay",
    0x1501E0: "TrailBlazerRender__BuildTrail",
    0x151880: "TrailBlazerRender__AddSegment",
    0x151DC0: "TrailBlazerRender__AddTrail",
    0x151ED0: "TrailBlazerRender__NextState",
    0x151F10: "TrailBlazerRender__IsSettled",
    0x151F50: "TrailBlazerRender__SetState",
    0x152100: "TrailBlazerRender__ChangeState",
    0x1521E0: "TrailBlazerRender__ChangeLineState",
    0x152290: "TrailBlazerRender__BlockCtor",
    0x123450: "DangerParticles__Ctor",
    0x1234A0: "DangerParticles__SetLevel",
    0x123510: "DangerParticles__Start",
    0x1235F0: "DangerParticles__Place",
    0x1238A0: "DangerParticles__Stop",
}
ok = sum(1 for a, n in N.items() if idc.set_name(a, n, idc.SN_NOWARN | idc.SN_NOCHECK))
print("renamed", ok, "of", len(N))
