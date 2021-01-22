use jni::{
    JNIEnv,
    objects::{
        JClass,
        JString,
    },
    sys::jstring,
};

use libcli;

#[no_mangle]
pub unsafe extern "system" fn Java_com_playerapp_PlayerWalletModule_hello(env: JNIEnv, _: JClass, name: JString) -> jstring { 
    let name: String = env.get_string(name).unwrap().into();
// 10.0.2.2 is for testing in the android studio emulator
    let LOCALHOST = "10.0.2.2";
    let conf = libcli::Conf {
        electrs_url: format!("tcp://{}:60401", LOCALHOST),
        name_url: format!("http://{}:18420", LOCALHOST),
        arbiter_url: format!("http://{}:5000", LOCALHOST),
    };
    let response = libcli::cli(name, conf);
    env.new_string(response).unwrap().into_inner()
}
