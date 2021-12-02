package io.github.doukutsu_rs;

import android.app.AlertDialog;
import android.app.NativeActivity;
import android.content.Intent;
import android.content.res.Configuration;
import android.hardware.SensorManager;
import android.os.Build;
import android.os.Bundle;
import android.view.OrientationEventListener;
import android.view.WindowInsets;

import java.io.File;

import static android.os.Build.VERSION.SDK_INT;

public class MainActivity extends NativeActivity {
    private int[] displayInsets = new int[]{0, 0, 0, 0};
    private OrientationEventListener listener;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        File f = new File(getFilesDir().getAbsolutePath() + "/data/");
        String[] list = f.list();
        if (!f.exists() || (list != null && list.length == 0)) {

            messageBox("Missing data files", "No data files found, would you like to download them?", () -> {
                Intent intent = new Intent(this, DownloadActivity.class);
                intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_TASK_ON_HOME);
                startActivity(intent);
                this.finish();
            });
        }

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

    private void messageBox(String title, String message, Runnable callback) {
        this.runOnUiThread(() -> {
            AlertDialog.Builder alert = new AlertDialog.Builder(this);
            alert.setTitle(title);
            alert.setMessage(message);
            alert.setPositiveButton(android.R.string.yes, (dialog, whichButton) -> {
                callback.run();
            });
            alert.setNegativeButton(android.R.string.no, (dialog, whichButton) -> {
                // hide
            });
            alert.show();
        });
    }
}
