package io.github.doukutsu_rs;

import android.app.Activity;

import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;

public class ActivityUtils {
    public static void hideSystemBars(Activity activity) {
        var window = activity.getWindow();
        var windowInsetsController =
                WindowCompat.getInsetsController(window, window.getDecorView());
        windowInsetsController.hide(WindowInsetsCompat.Type.systemBars());
    }
}
