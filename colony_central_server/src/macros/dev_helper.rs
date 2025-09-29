



// compile time warnings do not exist in rust, use deprecated as a workaround
#[cfg(feature = "dev")]
#[deprecated(note = "dev_todo!() is present in code — remove before release")]
#[macro_export]
macro_rules! dev_todo {
    () => { };
    ($($arg:tt)*) => {  };
}

#[cfg(not(feature = "dev"))]
macro_rules! dev_todo {
    () => { compile_error!("Found dev_todo!() in non-dev build"); };
    ($($arg:tt)*) => { compile_error!("Found dev_todo!() in non-dev build"); };
}











