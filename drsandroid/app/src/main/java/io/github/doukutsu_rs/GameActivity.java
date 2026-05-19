package io.github.doukutsu_rs;

import static android.os.Build.VERSION.SDK_INT;

import android.content.ActivityNotFoundException;
import android.content.Intent;
import android.content.res.Configuration;
import android.os.Build;
import android.provider.DocumentsContract;
import android.view.DisplayCutout;
import android.view.WindowInsets;
import android.widget.Toast;

import java.util.Arrays;
import java.io.File;

public class GameActivity extends org.libsdl.app.SDLActivity {
    private int[] displayInsets = new int[]{0, 0, 0, 0};

    @Override
    protected String[] getLibraries() {
        return new String[] {
            "SDL2",
            "drsandroid"
        };
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
        Arrays.fill(this.displayInsets, 0);

        WindowInsets insets = getWindow().getDecorView().getRootWindowInsets();
        if (insets != null) {
            this.displayInsets[0] = Math.max(this.displayInsets[0], insets.getStableInsetLeft());
            this.displayInsets[1] = Math.max(this.displayInsets[1], insets.getStableInsetTop());
            this.displayInsets[2] = Math.max(this.displayInsets[2], insets.getStableInsetRight());
            this.displayInsets[3] = Math.max(this.displayInsets[3], insets.getStableInsetBottom());
        }

        if (SDK_INT >= Build.VERSION_CODES.P) {
            DisplayCutout cutout = insets.getDisplayCutout();
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
