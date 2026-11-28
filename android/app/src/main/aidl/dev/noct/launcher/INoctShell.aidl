package dev.noct.launcher;

interface INoctShell {
    String recentTasks() = 1;
    boolean switchTo(int taskId) = 2;
    String lastError() = 3;
    void destroy() = 16777114;
}
