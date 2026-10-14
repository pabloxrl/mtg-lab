package org.mage.test.mtglab;

import java.lang.management.ManagementFactory;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.nio.file.StandardOpenOption;

/** Timing shell only: delegates unchanged real-AI capture and its controls. */
public final class CampaignCapture {
    public static void main(String[] args) throws Exception {
        long uptime = ManagementFactory.getRuntimeMXBean().getUptime();
        long start = System.nanoTime();
        boolean passed = false;
        try {
            new RawAiCaptureTest().capture();
            passed = true;
        } catch (Throwable error) {
            error.printStackTrace();
        } finally {
            double seconds = (System.nanoTime() - start) / 1e9;
            String json = "{\"jvm_start_to_entry_seconds\":" + uptime / 1000.0
                + ",\"capture_and_equivalence_seconds\":" + seconds
                + ",\"passed\":" + passed + "}\n";
            Files.write(Paths.get(System.getProperty("mtglab.output"), "java-timing.json"),
                json.getBytes(StandardCharsets.UTF_8), StandardOpenOption.CREATE_NEW);
        }
        System.exit(passed ? 0 : 1);
    }
}
