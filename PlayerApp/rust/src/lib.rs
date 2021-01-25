use jni::{
    JNIEnv,
    objects::{
        JClass,
        JString,
    },
    sys::jstring,
};
use libcli;

const LOCALHOST: &'static str  = "10.0.2.2";

#[no_mangle]
pub unsafe extern "system" fn Java_com_playerapp_PlayerWalletModule_cli(env: JNIEnv, _: JClass, command: JString) -> jstring { 
    let command: String = env.get_string(command).unwrap().into();
// 10.0.2.2 is for testing in the android studio emulator
    let conf = libcli::Conf {
        electrum_url: format!("tcp://{}:60401", LOCALHOST),
        name_url: format!("http://{}:18420", LOCALHOST),
        arbiter_url: format!("http://{}:5000", LOCALHOST),
    };
    let response = libcli::cli(command, conf);
    env.new_string(response).unwrap().into_inner()
}
