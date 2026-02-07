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

public class GameActivity extends org.libsdl.app.SDLActivity {
    @Override
    protected String[] getLibraries() {
        return new String[] {
            "SDL2",
            "drsandroid"
        };
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
