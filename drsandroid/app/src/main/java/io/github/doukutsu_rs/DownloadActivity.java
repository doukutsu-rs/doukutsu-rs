package io.github.doukutsu_rs;

import android.content.Intent;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.widget.ProgressBar;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;

import java.io.*;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.ArrayList;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import dalvik.system.ZipPathValidator;

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
        ActivityUtils.hideSystemBars(this);

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
        private final ArrayList<DownloadEntry> urls = new ArrayList<>();

        private final ArrayList<String> filesWhitelist = new ArrayList<>();

        @Override
        public void run() {
            this.filesWhitelist.add("data/");
            this.filesWhitelist.add("Doukutsu.exe");

            // DON'T SET `true` VALUE FOR TRANSLATIONS
            this.urls.add(new DownloadEntry(R.string.download_entries_base, "https://www.cavestory.org/downloads/cavestoryen.zip", true));

            for (DownloadEntry entry : this.urls) {
                this.download(entry);
            }
        }

        private void download(DownloadEntry downloadEntry) {
            HttpURLConnection connection = null;
            try {
                URL url = new URL(downloadEntry.url);
                connection = (HttpURLConnection) url.openConnection();
                connection.connect();

                if (connection.getResponseCode() != HttpURLConnection.HTTP_OK) {
                    throw new IllegalStateException(getString(R.string.download_status_error_http, connection.getResponseCode()));
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
                                    ? getString(R.string.download_status_downloading, downloadEntry.name, downloaded * 100 / fileLength, downloaded / 1024, fileLength / 1024, speed)
                                    : getString(R.string.download_status_downloading_null, downloadEntry.name, downloaded / 1024, speed);

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
                this.unpack(zipFile, downloadEntry.isBase);

                handler.post(() -> txtProgress.setText(getString(R.string.download_status_done)));

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

        private void unpack(byte[] zipFile, boolean isBase) throws IOException {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                ZipPathValidator.clearCallback();
            }
            ZipInputStream in = new ZipInputStream(new ByteArrayInputStream(zipFile));
            ZipEntry entry;
            byte[] buffer = new byte[4096];
            while ((entry = in.getNextEntry()) != null) {
                String entryName = entry.getName();

                // https://developer.android.com/privacy-and-security/risks/zip-path-traversal
                if (entryName.contains("..") || entryName.startsWith("/")) {
                    in.closeEntry();
                    continue;
                }

                // strip prefix
                if (entryName.startsWith("CaveStory/")) {
                    entryName = entryName.substring("CaveStory/".length());
                }

                if (!this.entryInWhitelist(entryName)) {
                    in.closeEntry();
                    continue;
                }


                final String s = entryName;
                handler.post(() -> txtProgress.setText(
                        getString(R.string.download_status_unpacking, s)
                ));

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

        private boolean entryInWhitelist(String entry) {
            for (String file : this.filesWhitelist) {
                if (entry.startsWith(file)) {
                    return true;
                }
            }

            return false;
        }
    }

    private class DownloadEntry {
        public String name; //e.g. "Polish translation", "Base data files"
        public String url;
        public boolean isBase = false;

        DownloadEntry(String name, String url, boolean isBase) {
            this.name = name;
            this.url = url;
            this.isBase = isBase;
        }

        DownloadEntry(int name, String url, boolean isBase) {
            this.name = getString(name);
            this.url = url;
            this.isBase = isBase;
        }
    }
}
