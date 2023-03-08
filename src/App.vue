<template>
  <router-view />
</template>

<script lang="ts" setup>
import {invoke} from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";

appWindow.listen("stream-added", ()=>{
  console.log("stream added")
})

appWindow.listen("stream-removed", ()=>{
  console.log("stream removed")
})

invoke("get_config").then(console.log)
</script>
