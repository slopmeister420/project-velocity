import idc
# names found in the 2026-11-07 coverage porting stage
for a, n in [(0x1813C0, "TriggerUpdate_DebugRenderTrigger"), (0xC1480, "CFEGameVariables::CFEGameVariables"),
             (0x175870, "CTimelineSequencer::LoadModels"), (0x176640, "CTimelineSequencer::BindAnims"),
             (0x174930, "CTimelineSequencer::FindAnim"), (0x9D860, "GDL_LightNameIds"),
             (0xA0B40, "GDL_LoadLights"), (0xA0080, "GDL_LoadAnim")]:
    print("%X %s" % (a, idc.set_name(a, n, idc.SN_NOWARN | idc.SN_NOCHECK)))
