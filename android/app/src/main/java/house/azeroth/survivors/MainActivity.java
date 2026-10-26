package house.azeroth.survivors;

import android.app.GameManager;
import android.app.GameState;
import android.content.SharedPreferences;
import android.os.Build;
import android.os.Bundle;
import android.os.PerformanceHintManager;
import android.os.Process;
import android.util.Log;
import android.view.Choreographer;
import android.view.Display;
import android.view.Surface;
import android.view.SurfaceView;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

import com.google.androidgamesdk.GameActivity;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;

public class MainActivity extends GameActivity {
    private static final String TAG = "AzerothSurvivors";
    private static final String PREFS = "benilla";
    private static final String PREF_MAX_FPS = "maxFps";

    static {
        System.loadLibrary("warcraft_survivors");
    }

    private PerformanceHintManager.Session hintSession;
    private long frameStartNs;
    private boolean hintsRunning;
    private int targetFps = 30;
    private int pollCounter;
    private final Choreographer.FrameCallback frameCb = new Choreographer.FrameCallback() {
        @Override
        public void doFrame(long frameTimeNanos) {
            if (!hintsRunning) {
                return;
            }
            if (hintSession != null && frameStartNs != 0) {
                long dt = frameTimeNanos - frameStartNs;
                if (dt > 0 && dt < 100_000_000L) {
                    hintSession.reportActualWorkDuration(dt);
                }
            }
            frameStartNs = frameTimeNanos;
            maybePollMaxFps();
            Choreographer.getInstance().postFrameCallback(this);
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Window window = getWindow();
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        if (Build.VERSION.SDK_INT >= 24) {
            window.setSustainedPerformanceMode(true);
        }
        int saved = getSharedPreferences(PREFS, MODE_PRIVATE).getInt(PREF_MAX_FPS, 30);
        pinTargetFps(saved);
        hideSystemUi();
        markGameplay();
        startAdpf();
    }

    @Override
    protected void onResume() {
        super.onResume();
        hintsRunning = true;
        frameStartNs = 0;
        pinTargetFps(targetFps);
        Choreographer.getInstance().postFrameCallback(frameCb);
    }

    @Override
    protected void onPause() {
        hintsRunning = false;
        super.onPause();
    }

    @Override
    protected void onDestroy() {
        if (hintSession != null) {
            hintSession.close();
            hintSession = null;
        }
        super.onDestroy();
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            pinTargetFps(targetFps);
            hideSystemUi();
        }
    }

    @Override
    public void onConfigurationChanged(android.content.res.Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        pinTargetFps(targetFps);
        hideSystemUi();
    }

    /** Called from native via the max-fps file; also from onCreate. */
    public void setTargetFps(int fps) {
        pinTargetFps(fps);
    }

    /**
     * Pick the same-resolution Display.Mode closest to {@code fps} (S22 Ultra 1440×3088:
     * mode 5 = 30 Hz, mode 3 = 60, mode 1 = 120). Then tell the compositor and ADPF.
     */
    private void pinTargetFps(int fps) {
        if (fps != 30 && fps != 60 && fps != 120) {
            fps = 30;
        }
        targetFps = fps;
        SharedPreferences prefs = getSharedPreferences(PREFS, MODE_PRIVATE);
        if (prefs.getInt(PREF_MAX_FPS, -1) != fps) {
            prefs.edit().putInt(PREF_MAX_FPS, fps).apply();
        }
        Window window = getWindow();
        WindowManager.LayoutParams lp = window.getAttributes();
        Display display = getDisplay();
        if (display == null) {
            return;
        }
        Display.Mode current = display.getMode();
        Display.Mode best = current;
        float bestErr = Float.MAX_VALUE;
        for (Display.Mode mode : display.getSupportedModes()) {
            if (mode.getPhysicalWidth() != current.getPhysicalWidth()
                    || mode.getPhysicalHeight() != current.getPhysicalHeight()) {
                continue;
            }
            float err = Math.abs(mode.getRefreshRate() - fps);
            boolean better =
                    err < bestErr - 0.05f
                    || (Math.abs(err - bestErr) <= 0.05f
                            && mode.getRefreshRate() + 0.05f >= fps
                            && best.getRefreshRate() + 0.05f < fps);
            if (better) {
                best = mode;
                bestErr = err;
            }
        }
        lp.preferredDisplayModeId = best.getModeId();
        lp.preferredRefreshRate = fps;
        window.setAttributes(lp);
        if (Build.VERSION.SDK_INT >= 30) {
            window.setPreferMinimalPostProcessing(true);
        }
        applyFrameRate(window.getDecorView(), fps);
        if (hintSession != null && Build.VERSION.SDK_INT >= 31) {
            long periodNs = (long) (1_000_000_000.0 / fps);
            hintSession.updateTargetWorkDuration(periodNs);
        }
        Log.i(TAG, "maxFps " + fps + " -> mode " + best.getModeId()
                + " " + best.getPhysicalWidth() + "x" + best.getPhysicalHeight()
                + " @" + best.getRefreshRate() + "Hz");
    }

    private void applyFrameRate(View v, float fps) {
        if (v == null) {
            return;
        }
        if (Build.VERSION.SDK_INT >= 35) {
            v.setRequestedFrameRate(fps);
        }
        if (v instanceof SurfaceView && Build.VERSION.SDK_INT >= 30) {
            Surface s = ((SurfaceView) v).getHolder().getSurface();
            if (s != null && s.isValid()) {
                s.setFrameRate(fps, Surface.FRAME_RATE_COMPATIBILITY_FIXED_SOURCE);
            }
        }
        if (v instanceof ViewGroup) {
            ViewGroup g = (ViewGroup) v;
            for (int i = 0; i < g.getChildCount(); i++) {
                applyFrameRate(g.getChildAt(i), fps);
            }
        }
    }

    /** Native writes {@code $BENILLA_HOME/max-fps} when Options changes the CVar. */
    private void maybePollMaxFps() {
        if (++pollCounter < 45) {
            return;
        }
        pollCounter = 0;
        File dir = getExternalFilesDir(null);
        if (dir == null) {
            return;
        }
        File f = new File(dir, "benilla-config/max-fps");
        if (!f.isFile()) {
            return;
        }
        try (BufferedReader r = new BufferedReader(new FileReader(f))) {
            String line = r.readLine();
            if (line == null) {
                return;
            }
            int hz = Integer.parseInt(line.trim());
            if (hz != targetFps && (hz == 30 || hz == 60 || hz == 120)) {
                pinTargetFps(hz);
            }
        } catch (Exception ignored) {
            // malformed line: keep the current pin
        }
    }

    /** ADPF: tell the 8 Gen 1 the frame budget (33.3 / 16.7 / 8.33 ms). */
    private void startAdpf() {
        if (Build.VERSION.SDK_INT < 31) {
            return;
        }
        PerformanceHintManager mgr = getSystemService(PerformanceHintManager.class);
        if (mgr == null) {
            return;
        }
        long periodNs = (long) (1_000_000_000.0 / Math.max(targetFps, 30));
        hintSession = mgr.createHintSession(new int[] {Process.myTid()}, periodNs);
    }

    private void hideSystemUi() {
        Window window = getWindow();
        window.setDecorFitsSystemWindows(false);
        WindowInsetsController controller = window.getInsetsController();
        if (controller != null) {
            controller.hide(WindowInsets.Type.statusBars() | WindowInsets.Type.navigationBars());
            controller.setSystemBarsBehavior(
                    WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
        }
    }

    private void markGameplay() {
        if (Build.VERSION.SDK_INT < 33) {
            return;
        }
        GameManager gm = getSystemService(GameManager.class);
        if (gm != null) {
            gm.setGameState(new GameState(true, GameState.MODE_GAMEPLAY_INTERRUPTIBLE));
        }
    }
}
