import idc
# core stage 7: replay camera bridge writes, camera bridge swap, triage
R = [
    (0x164D70, "ReplayCameraFixed::WriteBridge"),
    (0x1674E0, "ReplayCameraOrbit::WriteBridge"),
    (0x167D60, "ReplayCameraWheel::WriteBridge"),
    (0x969B0, "CameraBridge::SwapViews"),
    (0x169BF0, "ScoreManager::CreateScoreOsdItem"),
    (0x9FC90, "GDL_RelocateAnim"),
]
for ea, n in R:
    print(hex(ea), n, idc.set_name(ea, n, idc.SN_NOWARN | idc.SN_NOCHECK))
