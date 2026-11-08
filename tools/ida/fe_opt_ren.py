# frontend agent: name the options screens and the overlay menu functions
import idc
names = {
 0x2396B0: "FECK2CreateOptionsScreen", 0x239E40: "CFEOptionsScreen__OnEnter", 0x239EB0: "CFEOptionsScreen__OnActivate", 0x23A190: "CFEOptionsScreen__OnSelect",
 0x236290: "FECK2CreateVideoOptionsScreen", 0x236F80: "CFEVideoOptionsScreen__Refresh",
 0x2372C0: "FECK2CreateAudioOptionsScreen", 0x238170: "CFEAudioOptionsScreen__CycleSoundtrack", 0x238330: "CFEAudioOptionsScreen__ShowPicker",
 0x238510: "FECK2CreateConfigOptionsScreen", 0x239340: "CFEConfigOptionsScreen__CycleMapping", 0x239430: "CFEConfigOptionsScreen__Relabel",
 0x230AC0: "FECK2CreateLanguageOptionsScreen", 0x2316C0: "CFELanguageOptionsScreen__OnSelect", 0x231580: "CFELanguageOptionsScreen__OnUpdate",
 0x27D860: "CFE2DOverlay__MenuMetrics", 0x27DB00: "CFE2DOverlay__MenuWindow", 0x27DC10: "CFE2DOverlay__AddMenuEntry", 0x27E3E0: "CFE2DOverlay__AddMenuIconEntry",
 0x27EC70: "CFE2DOverlay__ClearMenu", 0x27EE60: "CFE2DOverlay__ShowMenu", 0x27F390: "CFE2DOverlay__MoveMenuSelection", 0x27F840: "CFE2DOverlay__LayoutMenu",
 0x27FD80: "CFE2DOverlay__LayoutMenuIcons", 0x280230: "CFE2DOverlay__SetMenuSelection", 0x280410: "CFE2DOverlay__SetMenuEntryEnabled",
 0x2808C0: "CFE2DOverlay__SetMenuWindow", 0x280B60: "CFE2DOverlay__SlideMenuBorder", 0x280E60: "CFE2DOverlay__SetMenuStyle", 0x280EC0: "CFE2DOverlay__PlaceMenu",
 0x276C70: "CFE2DOverlay__AssignMenuLayerEvents", 0x276F10: "CFE2DOverlay__AssignOverlayLayerEvents", 0x272140: "CFE3DCamera__SetFOV",
 0x2EBD70: "CStringManager__ChangeLanguage",
}
n = 0
for a, s in names.items():
    if idc.set_name(a, s, idc.SN_NOWARN | 0x800):
        n += 1
print("renamed", n)
