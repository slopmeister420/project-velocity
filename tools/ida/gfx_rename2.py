import idc
r = [(0x15A470, "CFogRender::Init"), (0x15A4E0, "CFogRender::SetFromSetup"), (0x15A5C0, "CFogRender::AddTweaks"),
     (0x15A990, "CFogRender::SetRenderStates"), (0x15ADB0, "CFogRender::Disable"),
     (0x9E300, "GDL_LoadTrackSetup"), (0x9DA10, "GDL_LoadPointLightsBin"), (0x9DE90, "GDL_LoadPointLightsTxt"),
     (0x59140, "VsRender::ComputeObjectLights"), (0x59050, "VsRender::SetPointLight"), (0x58F70, "VsRender::AllocPointLights"),
     (0x13BE20, "CSkyRender::UpdateClouds"), (0x13B5D0, "CSkyRender::Load"), (0x13C390, "CSkyRender::RenderSunMoon"),
     (0x2A3320, "CShaderManager::CompileStage"), (0x2A04C0, "CShaderManagerBase::CreateShader"),
     (0x2A0E40, "CShaderManagerBase::SetStageFlags"), (0x2A0EC0, "CShaderManagerBase::SetBlendMode"),
     (0x2A13B0, "CShaderManagerBase::SetZState"), (0x2A1570, "CShaderManagerBase::SetAddressWrap"),
     (0x2A1430, "CShaderManagerBase::SetTexture"), (0x2A1E80, "CShaderManagerBase::SetColourOp"),
     (0x2A1F40, "CShaderManagerBase::SetAlphaOp"), (0x2A2000, "CShaderManagerBase::SetStageArgs")]
for a, n in r:
    print(hex(a), idc.set_name(a, n, idc.SN_NOWARN | idc.SN_NOCHECK))
