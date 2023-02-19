<template>
  <router-view />
</template>

<script lang="ts" setup>
import {invoke} from "@tauri-apps/api/tauri";

invoke("get_config").then(console.log)
</script>
