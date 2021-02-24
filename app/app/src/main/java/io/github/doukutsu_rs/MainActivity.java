package io.github.doukutsu_rs;

import android.app.NativeActivity;
import android.content.res.Configuration;
import android.hardware.SensorManager;
import android.os.Build;
import android.os.Bundle;
import android.view.OrientationEventListener;
import android.view.WindowInsets;

import static android.os.Build.VERSION.SDK_INT;

public class MainActivity extends NativeActivity {
    private int[] displayInsets = new int[]{0, 0, 0, 0};
    private OrientationEventListener listener;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        listener = new OrientationEventListener(this, SensorManager.SENSOR_DELAY_UI) {
            @Override
            public void onOrientationChanged(int orientation) {
                MainActivity.this.updateCutouts();
            }
        };

        if (listener.canDetectOrientation()) {
            listener.enable();
        } else {
            listener = null;
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();

        if (listener != null) {
            listener.disable();

            listener = null;
        }
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
        if (SDK_INT >= Build.VERSION_CODES.P) {
            WindowInsets insets = getWindow().getDecorView().getRootWindowInsets();

            if (insets != null) {
                android.view.DisplayCutout cutout = insets.getDisplayCutout();

                if (cutout != null) {
                    this.displayInsets[0] = cutout.getSafeInsetLeft();
                    this.displayInsets[1] = cutout.getSafeInsetTop();
                    this.displayInsets[2] = cutout.getSafeInsetRight();
                    this.displayInsets[3] = cutout.getSafeInsetBottom();
                }
            }
        }
    }
}
