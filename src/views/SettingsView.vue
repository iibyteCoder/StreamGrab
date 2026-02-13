<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();
</script>

<template>
  <div class="h-full overflow-auto p-6">
    <div class="mb-6 flex items-center gap-4">
      <button
        @click="router.push('/')"
        class="text-muted-foreground hover:text-foreground transition-colors"
      >
        ← 返回
      </button>
      <h1 class="text-2xl font-semibold">设置</h1>
    </div>

    <div class="mx-auto max-w-2xl space-y-6">
      <!-- Download Settings -->
      <section class="rounded-lg border border-border p-4">
        <h2 class="mb-4 text-lg font-medium">下载设置</h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="text-sm">下载线程数</label>
            <input
              type="number"
              value="16"
              class="w-20 rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            />
          </div>
          <div class="flex items-center justify-between">
            <label class="text-sm">重试次数</label>
            <input
              type="number"
              value="3"
              class="w-20 rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            />
          </div>
        </div>
      </section>

      <!-- General Settings -->
      <section class="rounded-lg border border-border p-4">
        <h2 class="mb-4 text-lg font-medium">常规设置</h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="text-sm">保存目录</label>
            <div class="flex gap-2">
              <input
                type="text"
                value="./output"
                class="w-48 rounded-md border border-input bg-background px-3 py-1.5 text-sm"
              />
              <button class="rounded-md border border-input px-3 py-1.5 text-sm hover:bg-accent">
                浏览
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- Advanced Settings -->
      <section class="rounded-lg border border-border p-4">
        <h2 class="mb-4 text-lg font-medium">高级设置</h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="text-sm">代理设置</label>
            <input
              type="text"
              placeholder="http://127.0.0.1:7890"
              class="w-48 rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
