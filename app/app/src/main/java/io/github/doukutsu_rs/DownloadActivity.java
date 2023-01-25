package io.github.doukutsu_rs;

import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import androidx.appcompat.app.AppCompatActivity;
import android.widget.ProgressBar;
import android.widget.TextView;

import java.io.*;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.Locale;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public class DownloadActivity extends AppCompatActivity {
    private TextView txtProgress;
    private ProgressBar progressBar;
    private DownloadThread downloadThread;
    private String basePath;
    private Handler handler = new Handler();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        setContentView(R.layout.activity_download);
        txtProgress = findViewById(R.id.txtProgress);
        progressBar = findViewById(R.id.progressBar);
        basePath = getFilesDir().getAbsolutePath() + "/";

        downloadThread = new DownloadThread();
        downloadThread.start();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        downloadThread.interrupt();
    }

    private class DownloadThread extends Thread {
        private static final String DOWNLOAD_URL = "https://www.cavestory.org/downloads/cavestoryen.zip";

        @Override
        public void run() {
            HttpURLConnection connection = null;
            try {
                URL url = new URL(DOWNLOAD_URL);
                connection = (HttpURLConnection) url.openConnection();
                connection.connect();

                if (connection.getResponseCode() != HttpURLConnection.HTTP_OK) {
                    throw new IllegalStateException("Bad HTTP response code: " + connection.getResponseCode());
                }

                int fileLength = connection.getContentLength();
                if (fileLength == 0) {
                    handler.post(() -> progressBar.setIndeterminate(true));
                }

                byte[] zipFile;
                {
                    InputStream input = new BufferedInputStream(connection.getInputStream());
                    ByteArrayOutputStream output = new ByteArrayOutputStream();

                    int downloadedLast = 0;
                    int downloaded = 0;
                    byte[] buffer = new byte[4096];
                    int count;
                    long last = System.currentTimeMillis();

                    while ((count = input.read(buffer)) != -1) {
                        downloaded += count;

                        output.write(buffer, 0, count);

                        long now = System.currentTimeMillis();
                        if (last + 1000 >= now) {
                            int speed = (int) ((downloaded - downloadedLast) / 1024.0);
                            String text = (fileLength > 0)
                                    ? String.format(Locale.ENGLISH, "Downloading... %d%% (%d/%d KiB, %d KiB/s)", downloaded * 100 / fileLength, downloaded / 1024, fileLength / 1024, speed)
                                    : String.format(Locale.ENGLISH, "Downloading... --%% (%d KiB, %d KiB/s)", downloaded / 1024, speed);

                            handler.post(() -> txtProgress.setText(text));

                            downloadedLast = downloaded;
                            last = now;
                        }
                    }

                    output.flush();
                    zipFile = output.toByteArray();
                    output.close();
                }

                new File(basePath).mkdirs();
                try (ZipInputStream in = new ZipInputStream(new ByteArrayInputStream(zipFile))) {
                    ZipEntry entry;
                    byte[] buffer = new byte[4096];
                    while ((entry = in.getNextEntry()) != null) {
                        String entryName = entry.getName();

                        // strip prefix
                        if (entryName.startsWith("CaveStory/")) {
                            entryName = entryName.substring("CaveStory/".length());
                        }

                        final String s = entryName;
                        handler.post(() -> txtProgress.setText("Unpacking: " + s));

                        if (entry.isDirectory()) {
                            new File(basePath + entryName).mkdirs();
                        } else {
                            try (FileOutputStream fos = new FileOutputStream(basePath + entryName)) {
                                int count;
                                while ((count = in.read(buffer)) != -1) {
                                    fos.write(buffer, 0, count);
                                }
                            }
                        }

                        in.closeEntry();
                    }
                }

                handler.post(() -> txtProgress.setText("Done!"));

                handler.post(() -> {
                    Intent intent = new Intent(DownloadActivity.this, GameActivity.class);
                    intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
                    startActivity(intent);
                    DownloadActivity.this.finish();
                });
            } catch (Exception e) {
                handler.post(() -> { 
                    if (txtProgress != null) 
                        txtProgress.setText(e.getMessage());
                });
                e.printStackTrace();
            } finally {
                if (connection != null) connection.disconnect();
            }
        }
    }
}
