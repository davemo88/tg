use jni::{
    JNIEnv,
    objects::{
        JClass,
        JString,
    },
    sys::jstring,
};

#[no_mangle]
pub unsafe extern "system" fn Java_com_playerapp_PlayerWalletModule_hello(env: JNIEnv, _: JClass, name: JString) -> jstring { 
    let name: String = env.get_string(name).unwrap().into();
    let response = format!("Hello from Rust, {}!", name);
    env.new_string(response).unwrap().into_inner()
}
