package io.github.doukutsu_rs;

import android.app.AlertDialog;
import android.content.Intent;
import android.os.Bundle;

import androidx.appcompat.app.AppCompatActivity;

import java.io.File;

public class MainActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        ActivityUtils.hideSystemBars(this);

        var f = new File(getFilesDir().getAbsolutePath() + "/data/");
        var list = f.list();
        if (!f.exists() || (list != null && list.length == 0)) {
            messageBox(getString(R.string.missing_data_title), getString(R.string.missing_data_desc), () -> {
                Intent intent = new Intent(this, DownloadActivity.class);
                intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
                startActivity(intent);
                this.finish();
            }, this::launchGame);
        } else {
            launchGame();
        }
    }

    private void launchGame() {
        var intent = new Intent(this, GameActivity.class);
        intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        startActivity(intent);
        this.finish();
    }

    private void messageBox(String title, String message, Runnable yesCallback, Runnable noCallback) {
        this.runOnUiThread(() -> {
            var alert = new AlertDialog.Builder(this);
            alert.setTitle(title);
            alert.setMessage(message);
            alert.setPositiveButton(android.R.string.yes, (dialog, whichButton) -> yesCallback.run());
            alert.setNegativeButton(android.R.string.no, (dialog, whichButton) -> noCallback.run());
            alert.setCancelable(false);
            alert.show();
        });
    }
}
