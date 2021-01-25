package com.playerapp;
import com.facebook.react.bridge.NativeModule;
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

    @ReactMethod
    public String call_cli(String command) {
        String cli_output = PlayerWalletModule.cli(command);
        Log.d("PlayerWalletModule", cli_output);
        return cli_output;
    }
}

