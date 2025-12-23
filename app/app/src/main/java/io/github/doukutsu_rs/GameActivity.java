package io.github.doukutsu_rs;

import android.app.NativeActivity;
import android.content.ActivityNotFoundException;
import android.content.Context;
import android.content.Intent;
import android.content.res.Configuration;
import android.hardware.SensorManager;
import android.hardware.input.InputManager;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.view.InputDevice;
import android.view.MotionEvent;
import android.view.OrientationEventListener;
import android.view.WindowInsets;
import android.widget.Toast;

import java.io.File;
import java.util.HashMap;
import java.util.Map;

import static android.os.Build.VERSION.SDK_INT;

public class GameActivity extends NativeActivity implements InputManager.InputDeviceListener {
    private int[] displayInsets = new int[]{0, 0, 0, 0};
    private OrientationEventListener listener;
    private InputManager inputManager;

    // Gamepad state - accessed via JNI from Rust
    // Format: [deviceId, buttons, leftX, leftY, rightX, rightY, triggerL, triggerR] per gamepad
    // Note: Only axis values and D-pad from HAT axis are read here.
    // Button events (A/B/X/Y etc.) go directly to native code via winit.
    // buttons field only contains D-pad bits: 11=Up, 12=Down, 13=Left, 14=Right
    // Axis values are scaled to int: -32767 to 32767
    public static final int MAX_GAMEPADS = 4;
    public static final int GAMEPAD_DATA_SIZE = 8;
    public volatile int[] gamepadData = new int[MAX_GAMEPADS * GAMEPAD_DATA_SIZE];
    public volatile int gamepadCount = 0;
    private Map<Integer, Integer> deviceIdToIndex = new HashMap<>();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        ActivityUtils.hideSystemBars(this);

        listener = new OrientationEventListener(this, SensorManager.SENSOR_DELAY_UI) {
            @Override
            public void onOrientationChanged(int orientation) {
                GameActivity.this.updateCutouts();
            }
        };

        if (listener.canDetectOrientation()) {
            listener.enable();
        } else {
            listener = null;
        }

        // Initialize gamepad support
        inputManager = (InputManager) getSystemService(Context.INPUT_SERVICE);
        inputManager.registerInputDeviceListener(this, null);
        scanForGamepads();
    }

    private void scanForGamepads() {
        int[] deviceIds = inputManager.getInputDeviceIds();
        for (int deviceId : deviceIds) {
            InputDevice device = inputManager.getInputDevice(deviceId);
            if (device != null && isGamepad(device)) {
                addGamepad(deviceId);
            }
        }
    }

    private boolean isGamepad(InputDevice device) {
        int sources = device.getSources();
        return ((sources & InputDevice.SOURCE_GAMEPAD) == InputDevice.SOURCE_GAMEPAD)
                || ((sources & InputDevice.SOURCE_JOYSTICK) == InputDevice.SOURCE_JOYSTICK);
    }

    private synchronized void addGamepad(int deviceId) {
        if (deviceIdToIndex.containsKey(deviceId)) {
            return; // Already added
        }
        if (gamepadCount >= MAX_GAMEPADS) {
            return; // No room
        }
        int index = gamepadCount++;
        deviceIdToIndex.put(deviceId, index);
        int base = index * GAMEPAD_DATA_SIZE;
        gamepadData[base] = deviceId;
        for (int i = 1; i < GAMEPAD_DATA_SIZE; i++) {
            gamepadData[base + i] = 0;
        }
    }

    private synchronized void removeGamepad(int deviceId) {
        Integer index = deviceIdToIndex.remove(deviceId);
        if (index == null) {
            return;
        }
        // Shift remaining gamepads down
        int base = index * GAMEPAD_DATA_SIZE;
        for (int i = index + 1; i < gamepadCount; i++) {
            int srcBase = i * GAMEPAD_DATA_SIZE;
            int dstBase = (i - 1) * GAMEPAD_DATA_SIZE;
            for (int j = 0; j < GAMEPAD_DATA_SIZE; j++) {
                gamepadData[dstBase + j] = gamepadData[srcBase + j];
            }
            // Update mapping
            int movedDeviceId = gamepadData[dstBase];
            deviceIdToIndex.put(movedDeviceId, i - 1);
        }
        gamepadCount--;
    }

    // InputDeviceListener callbacks
    @Override
    public void onInputDeviceAdded(int deviceId) {
        InputDevice device = inputManager.getInputDevice(deviceId);
        if (device != null && isGamepad(device)) {
            addGamepad(deviceId);
        }
    }

    @Override
    public void onInputDeviceRemoved(int deviceId) {
        removeGamepad(deviceId);
    }

    @Override
    public void onInputDeviceChanged(int deviceId) {
        // Treat as remove + add
        removeGamepad(deviceId);
        InputDevice device = inputManager.getInputDevice(deviceId);
        if (device != null && isGamepad(device)) {
            addGamepad(deviceId);
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();

        if (listener != null) {
            listener.disable();
            listener = null;
        }

        if (inputManager != null) {
            inputManager.unregisterInputDeviceListener(this);
        }
    }

    // Handle gamepad axis events (sticks, triggers, D-pad via HAT axis)
    // Note: Button events (A/B/X/Y etc.) bypass Java and go directly to native code,
    // where they are handled via winit's KeyboardInput events with Linux scancodes.
    @Override
    public boolean dispatchGenericMotionEvent(MotionEvent event) {
        int deviceId = event.getDeviceId();
        Integer index = deviceIdToIndex.get(deviceId);

        // Check if this is from a gamepad
        if (index == null) {
            InputDevice device = event.getDevice();
            if (device != null && isGamepad(device)) {
                addGamepad(deviceId);
                index = deviceIdToIndex.get(deviceId);
            }
        }

        if (index != null && (event.getSource() & InputDevice.SOURCE_JOYSTICK) == InputDevice.SOURCE_JOYSTICK) {
            int base = index * GAMEPAD_DATA_SIZE;

            // Left stick
            float leftX = event.getAxisValue(MotionEvent.AXIS_X);
            float leftY = event.getAxisValue(MotionEvent.AXIS_Y);

            // Right stick
            float rightX = event.getAxisValue(MotionEvent.AXIS_Z);
            float rightY = event.getAxisValue(MotionEvent.AXIS_RZ);

            // Triggers
            float triggerL = event.getAxisValue(MotionEvent.AXIS_LTRIGGER);
            float triggerR = event.getAxisValue(MotionEvent.AXIS_RTRIGGER);

            // Some controllers use BRAKE/GAS for triggers
            if (triggerL == 0) {
                triggerL = event.getAxisValue(MotionEvent.AXIS_BRAKE);
            }
            if (triggerR == 0) {
                triggerR = event.getAxisValue(MotionEvent.AXIS_GAS);
            }

            // Handle D-pad as axis (HAT_X, HAT_Y)
            float hatX = event.getAxisValue(MotionEvent.AXIS_HAT_X);
            float hatY = event.getAxisValue(MotionEvent.AXIS_HAT_Y);

            synchronized (this) {
                // Store axis values scaled to int range
                gamepadData[base + 2] = (int) (leftX * 32767);
                gamepadData[base + 3] = (int) (leftY * 32767);
                gamepadData[base + 4] = (int) (rightX * 32767);
                gamepadData[base + 5] = (int) (rightY * 32767);
                gamepadData[base + 6] = (int) (triggerL * 32767);
                gamepadData[base + 7] = (int) (triggerR * 32767);

                // Update D-pad from hat axis
                int buttons = gamepadData[base + 1];
                // Clear D-pad bits (11-14)
                buttons &= ~(0xF << 11);
                // Set D-pad from hat
                if (hatY < -0.5f) buttons |= (1 << 11); // Up
                if (hatY > 0.5f) buttons |= (1 << 12);  // Down
                if (hatX < -0.5f) buttons |= (1 << 13); // Left
                if (hatX > 0.5f) buttons |= (1 << 14);  // Right
                gamepadData[base + 1] = buttons;
            }

            // Consume the event
            return true;
        }

        return super.dispatchGenericMotionEvent(event);
    }


    @Override
    public void onAttachedToWindow() {
        super.onAttachedToWindow();

        this.updateCutouts();
    }

    @Override
    public void onConfigurationChanged(Configuration newConfig) {
        super.onConfigurationChanged(newConfig);

        this.updateCutouts();
    }

    private void updateCutouts() {
        this.displayInsets[0] = 0;
        this.displayInsets[1] = 0;
        this.displayInsets[2] = 0;
        this.displayInsets[3] = 0;

        WindowInsets insets = getWindow().getDecorView().getRootWindowInsets();

        if (insets != null) {
            this.displayInsets[0] = Math.max(this.displayInsets[0], insets.getStableInsetLeft());
            this.displayInsets[1] = Math.max(this.displayInsets[1], insets.getStableInsetTop());
            this.displayInsets[2] = Math.max(this.displayInsets[2], insets.getStableInsetRight());
            this.displayInsets[3] = Math.max(this.displayInsets[3], insets.getStableInsetBottom());
        } else {
            return;
        }

        if (SDK_INT >= Build.VERSION_CODES.P) {
            android.view.DisplayCutout cutout = insets.getDisplayCutout();

            if (cutout != null) {
                this.displayInsets[0] = Math.max(this.displayInsets[0], cutout.getSafeInsetLeft());
                this.displayInsets[1] = Math.max(this.displayInsets[0], cutout.getSafeInsetTop());
                this.displayInsets[2] = Math.max(this.displayInsets[0], cutout.getSafeInsetRight());
                this.displayInsets[3] = Math.max(this.displayInsets[0], cutout.getSafeInsetBottom());
            }
        }
    }

    public void openDir(String path) {
        Uri uri = DocumentsContract.buildDocumentUri(BuildConfig.DOCUMENTS_AUTHORITY, path);

        File file = new File(path);
        if (!file.isDirectory()) {
            Toast.makeText(getApplicationContext(), R.string.dir_not_found, Toast.LENGTH_LONG).show();
            return;
        }

        Intent intent = new Intent(Intent.ACTION_VIEW);
        intent.addCategory(Intent.CATEGORY_DEFAULT);
        intent.setDataAndType(uri, DocumentsContract.Document.MIME_TYPE_DIR);
        intent.setFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION | Intent.FLAG_GRANT_PREFIX_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);

        try {
            startActivity(intent);
        } catch(ActivityNotFoundException e) {
            Toast.makeText(getApplicationContext(), R.string.no_app_found_to_open_dir, Toast.LENGTH_LONG).show();
        }
    }
}
