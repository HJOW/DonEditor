package com.example.demo;

import java.util.ArrayList;
import java.util.List;

/**
 * 간단한 Java 예제 (Javadoc 주석).
 */
public class Sample<T extends Comparable<T>> implements Runnable {
    private static final int MAX_COUNT = 10;
    private final List<T> items = new ArrayList<>();
    private String name = "Don Editor";

    @Override
    public void run() {
        // 한 줄 주석
        for (int i = 0; i < MAX_COUNT; i++) {
            if (i % 2 == 0 && name != null) {
                System.out.println("값: " + i + ", 실수: " + 3.14f);
            }
        }
    }

    public <R> R convert(T value, java.util.function.Function<T, R> fn) {
        return fn.apply(value);
    }

    enum Level { LOW, HIGH }

    public static void main(String[] args) throws Exception {
        Sample<String> sample = new Sample<>();
        sample.items.add("a");
        boolean ok = true;
        char c = 'x';
        long big = 123_456L;
        try {
            sample.run();
        } catch (RuntimeException e) {
            throw new IllegalStateException(e);
        } finally {
            System.out.println(ok + " " + c + " " + big);
        }
    }
}
