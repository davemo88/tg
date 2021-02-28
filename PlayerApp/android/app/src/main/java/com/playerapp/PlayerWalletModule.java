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

    public static native String cli(String command);

    PlayerWalletModule(ReactApplicationContext context) {
        super(context);
    }

    @Override
    public String getName() {
        return "PlayerWalletModule";
    }

    @ReactMethod
    public void call_cli(String command, Promise promise) {
        String command_with_wallet_dir = add_wallet_dir(command);
        Log.d("PlayerWalletModule", "command: " + command_with_wallet_dir);
        String cli_output = PlayerWalletModule.cli(command_with_wallet_dir);
        Log.d("PlayerWalletModule", "cli output: " + cli_output);
        promise.resolve(cli_output);
    }

    @ReactMethod
    public void call_cli_with_passphrase(String command, String passphrase, Promise promise) {
        String command_with_wallet_dir = add_wallet_dir(command);
        Log.d("PlayerWalletModule", "command: " + command_with_wallet_dir + " --passphrase [redacted]");
        String cli_output = PlayerWalletModule.cli(command_with_wallet_dir + " --passphrase " + passphrase);
        Log.d("PlayerWalletModule", "cli output: " + cli_output);
        promise.resolve(cli_output);
    }

    private String add_wallet_dir(String command) {
        return command + " --wallet-dir " + getReactApplicationContext().getFilesDir().toString();
    }

}

