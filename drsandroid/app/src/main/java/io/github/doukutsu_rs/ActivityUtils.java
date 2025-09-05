package io.github.doukutsu_rs;

import android.app.Activity;
import android.view.Window;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

public class ActivityUtils {
    public static void hideSystemBars(Activity activity) {
        Window window = activity.getWindow();
        WindowInsetsControllerCompat windowInsetsController =
            WindowCompat.getInsetsController(window, window.getDecorView());
        windowInsetsController.hide(WindowInsetsCompat.Type.systemBars());
    }
}
