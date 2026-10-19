//! Run the real parameter setup through a minimal add-param host callback.
//! Its IDs/order must match the existing Windows v2.2 AE host report.
mod effect {
    include!("../src/lib.rs");
    pub fn check() {
        unsafe extern "C" fn add_param(
            ctx: ae::sys::PF_ProgPtr, _: ae::sys::PF_ParamIndex,
            param: ae::sys::PF_ParamDefPtr,
        ) -> ae::sys::PF_Err {
            let rows=unsafe { &mut *(ctx as *mut Vec<String>) };
            let p=unsafe { &*param };
            let name=unsafe { std::ffi::CStr::from_ptr(p.name_do_not_use_directly.as_ptr()) }.to_string_lossy();
            rows.push(format!("{name} (ChromaticAE RGB Split-{})", unsafe {p.uu.id}));
            0
        }
        let mut rows:Vec<String>=Vec::new();
        let mut input:ae::sys::PF_InData=unsafe {std::mem::zeroed()};
        let mut output:ae::sys::PF_OutData=unsafe {std::mem::zeroed()};
        input.effect_ref=(&mut rows as *mut Vec<String>).cast();
        input.inter.add_param=Some(add_param);
        let mut params=ae::Parameters::<Params>::new();params.set_in_data(&input);
        Plugin.params_setup(&mut params,ae::InData::from_raw(&input),ae::OutData::from_raw(&mut output)).unwrap();
        let expected:Vec<_>=include_str!("../../tests/native-results-26.5.txt").lines().filter(|s|s.contains(" (ChromaticAE RGB Split-")).collect();
        assert_eq!(rows,expected,"Saved control IDs or order changed across platforms");
        assert_eq!(rows.len(),32);
        println!("PASS all 32 saved control IDs and their order match Windows v2.2");
    }
}
fn main(){effect::check();}
