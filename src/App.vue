<script setup lang="ts">
import { RouterView } from "vue-router";
import { Toaster } from "@/components/ui/toast";
</script>

<template>
  <div class="h-screen w-screen overflow-hidden bg-background">
    <RouterView />
    <Toaster />
  </div>
</template>
