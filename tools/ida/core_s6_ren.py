import idc, ida_name
REN = {
    0x5E220: "VsBaseCamera::DebugRenderFrustum",
    0xBAF40: "GameManager::DebugFontTest",
    0xBF770: "GameManager::DebugLoadTimingAdd",
    0xBF910: "GameManager::DebugLoadTimingDump",
    0x16F860: "RaceTimelineSequencer::BindToTarget",
    0x171B90: "RaceTimelineData::LoadAnimations",
    0x171F20: "RaceTimelineData::LoadModels",
    0x170E50: "RaceTimelineData::Unload",
    0x172260: "RaceTimelineData::FindAnimation",
    0x172320: "RaceTimelineData::FindModel",
    0x2B6040: "TimelineSequencer::SeekTypedItem",
}
for a, n in REN.items():
    print("%X %s %s" % (a, n, idc.set_name(a, n, ida_name.SN_NOWARN | ida_name.SN_NOCHECK)))
