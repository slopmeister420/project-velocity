# frontend agent: names from the logo wall / steer stage (2026-11-07)
import idc
for ea, n in [
    (0x00225A80, "CFEQuickRaceCarSelectScreen__ShowTierLogos"),
    (0x00298690, "FECK2Element3D__SetMaterialTexture"),
    (0x0026F7A0, "FE3DCamera__GetBackground"),
    (0x0025E040, "FE3DVehicles__SetSteer"),
    (0x0025E7D0, "FE3DVehicles__UpdateHeights"),
    (0x0025EAB0, "FE3DVehicles__StartWheelBounce"),
    (0x0025B4D0, "FE3DVehicles__WheelMatrix"),
    (0x0025BA40, "FE3DVehicles__BrakeDiscMatrix"),
]:
    print(hex(ea), n, idc.set_name(ea, n, idc.SN_NOWARN | idc.SN_NOCHECK))
