package com.playerapp;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import java.util.Map;
import java.util.HashMap;
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
        Log.d("PlayerWalletModule", "command: " + command);
        ReactApplicationContext r = getReactApplicationContext();
        String filesDir = getReactApplicationContext().getFilesDir().toString();
        Log.d("PlayerWalletModule", "filesDir: " + filesDir);
        String command_with_wallet_path = command + " --wallet-path " + filesDir;
        Log.d("PlayerWalletModule", "command with wallet path: " + command_with_wallet_path);
        String cli_output = PlayerWalletModule.cli(command_with_wallet_path);
        Log.d("PlayerWalletModule", "cli output: " + cli_output);
        promise.resolve(cli_output);
    }

    @ReactMethod
    public void call_cli_bytes(ReadableArray command) {
//        byte[] cmd_bytes = new byte[command.size() * 4]
//        ReactApplicationContext r = getReactApplicationContext();
        Log.d("PlayerWalletModule", "command:");
        Log.d("PlayerWalletModule", command);
        ArrayList<Object> list = command.toArrayList();
        Lod.d("PlayerWalletModule", "list:");
        Lod.d("PlayerWalletModule", list);
        for (int i = 0; i < list.size(); i++) {
            Lod.d("PlayerWalletModule", list.get(i));
        }
    }
}

