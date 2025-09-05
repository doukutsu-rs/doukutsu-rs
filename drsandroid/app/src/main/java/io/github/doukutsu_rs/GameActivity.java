package io.github.doukutsu_rs;

import static android.os.Build.VERSION.SDK_INT;

import android.app.NativeActivity;
import android.content.ActivityNotFoundException;
import android.content.Intent;
import android.content.res.Configuration;
import android.hardware.SensorManager;
import android.os.Build;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.view.OrientationEventListener;
import android.widget.Toast;

import java.io.File;

public class GameActivity extends NativeActivity {
    private int[] displayInsets = new int[]{0, 0, 0, 0};
    private OrientationEventListener listener;

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

        var insets = getWindow().getDecorView().getRootWindowInsets();

        if (insets != null) {
            this.displayInsets[0] = Math.max(this.displayInsets[0], insets.getStableInsetLeft());
            this.displayInsets[1] = Math.max(this.displayInsets[1], insets.getStableInsetTop());
            this.displayInsets[2] = Math.max(this.displayInsets[2], insets.getStableInsetRight());
            this.displayInsets[3] = Math.max(this.displayInsets[3], insets.getStableInsetBottom());
        } else {
            return;
        }

        if (SDK_INT >= Build.VERSION_CODES.P) {
            var cutout = insets.getDisplayCutout();

            if (cutout != null) {
                this.displayInsets[0] = Math.max(this.displayInsets[0], cutout.getSafeInsetLeft());
                this.displayInsets[1] = Math.max(this.displayInsets[0], cutout.getSafeInsetTop());
                this.displayInsets[2] = Math.max(this.displayInsets[0], cutout.getSafeInsetRight());
                this.displayInsets[3] = Math.max(this.displayInsets[0], cutout.getSafeInsetBottom());
            }
        }
    }

    public void openDir(String path) {
        var uri = DocumentsContract.buildDocumentUri(BuildConfig.DOCUMENTS_AUTHORITY, path);

        var file = new File(path);
        if (!file.isDirectory()) {
            Toast.makeText(getApplicationContext(), R.string.dir_not_found, Toast.LENGTH_LONG).show();
            return;
        }

        var intent = new Intent(Intent.ACTION_VIEW);
        intent.addCategory(Intent.CATEGORY_DEFAULT);
        intent.setDataAndType(uri, DocumentsContract.Document.MIME_TYPE_DIR);
        intent.setFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION | Intent.FLAG_GRANT_PREFIX_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);

        try {
            startActivity(intent);
        } catch (ActivityNotFoundException e) {
            Toast.makeText(getApplicationContext(), R.string.no_app_found_to_open_dir, Toast.LENGTH_LONG).show();
        }
    }
}
