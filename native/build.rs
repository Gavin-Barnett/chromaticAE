use pipl::*;
fn main() {
    println!("cargo:rustc-cfg=catch_panics");
    pipl::plugin_build(vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("Chromatic AE"),
        Property::Category("Chromatic"),
        Property::CodeWin64X86("EffectMain"),
        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version {
            major: 13,
            minor: 28,
        },
        Property::AE_Effect_Version {
            version: 2,
            subversion: 2,
            bugversion: 0,
            stage: Stage::Release,
            build: 1,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(OutFlags::DeepColorAware | OutFlags::IExpandBuffer),
        Property::AE_Effect_Global_OutFlags_2(
            OutFlags2::FloatColorAware
                | OutFlags2::SupportsSmartRender
                | OutFlags2::SupportsThreadedRendering
                | OutFlags2::SupportsGetFlattenedSequenceData
                | OutFlags2::ParamGroupStartCollapsedFlag
                | OutFlags2::RevealsZeroAlpha,
        ),
        Property::AE_Effect_Match_Name("ChromaticAE RGB Split"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://github.com/Gavin-Barnett/chromaticAE"),
    ]);
}
