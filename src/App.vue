<template>
  <router-view />
</template>

<script lang="ts" setup>
import {invoke} from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import {useColorMode} from "@vueuse/core";
import {useTheme} from "vuetify";
import {watch} from "vue";

const mode = useColorMode()
const theme = useTheme()

watch(mode, (value) => {
  theme.global.name.value = value;
})

appWindow.listen("stream-added", ()=>{
  console.log("stream added")
})

appWindow.listen("stream-removed", ()=>{
  console.log("stream removed")
})

invoke("get_config").then(console.log)
</script>
