package com.playerapp;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import java.util.ArrayList;
import android.util.Log;

public class PlayerWalletModule extends ReactContextBaseJavaModule {
    static {
        System.loadLibrary("test");
    }

// need a new method using byte[] instead of string
    public static native String cli(String command);

    PlayerWalletModule(ReactApplicationContext context) {
        super(context);
    }

    @Override
    public String getName() {
        return "PlayerWalletModule";
    }

//TODO: this function should actually take a utf-8 byte array
// because String will get stuck in memory for too long
// react native provides ReadableArray, not sure how it works
// e.g. it may have the same garbage collection problem as String
// if i can zero the ReadableArray of utf-8 bytes, that's good
// i guess we can just store them as numbers since js has no bytes
// see e.g. https://devslash.net/why-you-dont-store-secrets-in-strings-in-java/
    @ReactMethod
    public void call_cli(String command, Promise promise) {
        Log.d("PlayerWalletModule", "command: " + redact_passphrase(command));
        ReactApplicationContext r = getReactApplicationContext();
        String filesDir = getReactApplicationContext().getFilesDir().toString();
        String command_with_wallet_path = command + " --wallet-path " + filesDir;
        Log.d("PlayerWalletModule", "command with wallet path: " + redact_passphrase(command_with_wallet_path));
        String cli_output = PlayerWalletModule.cli(command_with_wallet_path);
        Log.d("PlayerWalletModule", "cli output: " + cli_output);
        promise.resolve(cli_output);
    }

    private String redact_passphrase(String command) {
        String[] split = command.split("passphrase");
        if (split.length > 1) {
            String redacted = split[1].trim().split(" ")[0]; 
            return command.replaceAll(redacted,"[redacted]");
        } else {
            return command;
        }

    }

// TODO: this function will never do what I want since the best I can do
// is get the ArrayList<Object>, which at best will be a wrapper class like
// Integer or Double, and therefore immutable also stuck in memory
// I guess at least the individual elements will not be contiguous in memory
// since it's an array of Object so maybe that's worth something for foiling
// memory spies
//
// e.g. in React we could do
//     let utf8 = unescape(encodeURIComponent("trolol i am the best weeeeee"));
//     let arr = [];
//     for (let i = 0; i < utf8.length; i++) {
//         arr.push(utf8.charCodeAt(i));
//     }
//     PlayerWalletModule.call_cli_bytes(arr);
//
// getClass on the elements of the Java ArrayList returns Double
//
//    @ReactMethod
//    public void call_cli_bytes(ReadableArray command) {
//        ArrayList<Object> list = command.toArrayList();
//        Log.d("PlayerWalletModule", "original array");
//        Log.d("PlayerWalletModule", list.toString());
//        for (int i = 0; i < list.size(); i++) {
//            Log.d("PlayerWalletModule", list.get(i).getClass().getName());
//            list.set(i, 0);
//        }
//        Log.d("PlayerWalletModule", "zeroed array");
//        Log.d("PlayerWalletModule", list.toString());
//    }
}

