package com.playerapp;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import java.util.Map;
import java.util.HashMap;
import android.util.Log;

public class PlayerWalletModule extends ReactContextBaseJavaModule {
    static {
        System.loadLibrary("test");
    }

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
}

