// doc/testing/overview.md#failpoints
// doc/shared_libraries.md#clock-and-failpoints

#[cfg(feature = "failpoints")]
pub fn setup() {
    std::mem::forget(fail::FailScenario::setup());
}

#[cfg(not(feature = "failpoints"))]
pub fn setup() {}

#[cfg(feature = "failpoints")]
pub fn hit(name: &str) -> bool {
    match fail::eval(name, |arg| arg) {
        None => false,
        Some(Some(arg)) if arg == "abort" => std::process::abort(),
        Some(_) => true,
    }
}

#[cfg(not(feature = "failpoints"))]
#[inline(always)]
pub fn hit(_name: &str) -> bool {
    false
}
